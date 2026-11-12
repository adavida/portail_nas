#!/usr/bin/env bash
# Génère le fichier de config du backend depuis les variables d'env.
# usage: scripts/generate-config.sh [sqlite|ldap]   (défaut: sqlite, ou $DIRECTORY_BACKEND)
set -euo pipefail

BACKEND="${1:-${DIRECTORY_BACKEND:-sqlite}}"
case "$BACKEND" in
  sqlite | ldap) ;;
  *)
    echo "usage: $0 [sqlite|ldap]" >&2
    exit 1
    ;;
esac

STATE="${DEVENV_STATE:-$PWD/.devenv/state}"
OUT="${PORTAIL_CONFIG_FILE:-$STATE/config.toml}"
mkdir -p "$(dirname "$OUT")"
CLIENT_ID="${OIDC_CLIENT_ID:-portail-dev}"
REDIRECT_URI="${OIDC_REDIRECT_URI:-http://localhost:5173/callback}"

cat >"$OUT" <<TOML
app_url = "${APP_URL:-http://localhost:5173}"
bind_addr = "${BIND_ADDR:-0.0.0.0:3000}"
db_path = "${DB_PATH:-$STATE/db/portail.db}"
system_sensors = ["memory", "cpu", "network", "disk"]

[directory]
backend = "$BACKEND"

[ldap]
url = "${LDAP_URL:-ldaps://127.0.0.1:6360}"
base_dn = "${LDAP_BASE_DN:-dc=dev,dc=example,dc=com}"
tls_ca = "${LDAP_TLS_CA:-$STATE/openldap-dev/ldap.crt}"

[oidc]
issuer_url = "${OIDC_ISSUER_URL:-http://127.0.0.1:3000}"
client_id = "$CLIENT_ID"
redirect_uri = "$REDIRECT_URI"
secrets_dir = "${OIDC_SECRETS_DIR:-$STATE/secrets}"
clients = [
  { id = "$CLIENT_ID", secret_file = "", enabled = true, redirect_uris = ["$REDIRECT_URI"], scopes = ["openid", "profile", "email", "groups"] },
  { id = "oauth-debugger", enabled = false, redirect_uris = ["http://127.0.0.1:3030/callback"], scopes = ["openid", "profile", "email", "groups"] },
]
TOML

echo "wrote $OUT (directory backend: $BACKEND)"
