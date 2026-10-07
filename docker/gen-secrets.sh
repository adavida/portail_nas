#!/usr/bin/env bash
# Génère les secrets de la stack docker-compose dans docker/secrets/ + docker/.env.
# Idempotent : un fichier existant n'est jamais écrasé (supprimer le fichier pour régénérer).
# Usage : bash docker/gen-secrets.sh
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
dir="$here/secrets"
mkdir -p "$dir"

rand() { # rand <fichier> [octets-hex]
  [ -e "$dir/$1" ] || openssl rand -hex "${2:-32}" >"$dir/$1"
}

rand ldap-admin-pw 16
rand oidc-client-secret
rand session-secret
rand storage-encryption-key
rand reset-password-jwt-secret
rand hmac-secret

[ -e "$dir/jwks.pem" ] || openssl genrsa 2048 2>/dev/null >"$dir/jwks.pem"

# Cert TLS commun (Authelia https://authelia.localhost:8080 et nginx https://portail.localhost:8081).
if [ ! -e "$dir/portal.crt" ]; then
  openssl req -newkey rsa:4096 -nodes -x509 -days 3650 -sha256 \
    -keyout "$dir/portal.key" -out "$dir/portal.crt" \
    -subj "/CN=portail.localhost" \
    -addext "subjectAltName=DNS:portail.localhost,DNS:authelia.localhost,DNS:*.localhost,IP:127.0.0.1" 2>/dev/null
fi

if [ ! -e "$dir/oidc-client-secret-hash" ]; then
  secret="$(cat "$dir/oidc-client-secret")"
  hash_cmd=(authelia crypto hash generate pbkdf2 --password "$secret" --no-confirm)
  command -v authelia >/dev/null 2>&1 || hash_cmd=(docker run --rm docker.io/authelia/authelia:4 "${hash_cmd[@]}")
  "${hash_cmd[@]}" | tail -n 1 | sed 's/^Digest: //' >"$dir/oidc-client-secret-hash"
fi

chmod 644 "$dir"/*

[ -e "$here/.env" ] || printf 'LDAP_ADMIN_PASSWORD=%s\n' "$(cat "$dir/ldap-admin-pw")" >"$here/.env"

# Rendu de la config Authelia (les placeholders évitent {{ }} qui treefmt/prettier casserait).
awk -v pem="$(sed 's/^/          /' "$dir/jwks.pem")" -v hash="$(cat "$dir/oidc-client-secret-hash")" \
  '{ if ($0=="#JWKS_PEM#") print pem; else gsub("__CLIENT_SECRET_HASH__", hash); print }' \
  "$here/authelia/config.yml.template" >"$here/authelia/config.yml"

echo "secrets OK: $dir (+ $here/.env, $here/authelia/config.yml) — ne pas committer"
