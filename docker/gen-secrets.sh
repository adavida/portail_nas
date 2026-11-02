#!/usr/bin/env bash
# Génère les secrets de la stack docker-compose dans docker/secrets/ + docker/.env.
# Idempotent : un fichier existant n'est jamais écrasé (supprimer le fichier pour régénérer).
# Les secrets OIDC (clé de signature, clients.json) sont générés par le backend
# lui-même dans le volume `oidc-secrets` au premier start — rien ici.
# Usage : bash docker/gen-secrets.sh
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
dir="$here/secrets"
mkdir -p "$dir"

rand() { # rand <fichier> [octets-hex]
  [ -e "$dir/$1" ] || openssl rand -hex "${2:-32}" >"$dir/$1"
}

rand ldap-admin-pw 16

# Cert TLS nginx https://portail.localhost (le portail sert aussi l'OIDC).
if [ ! -e "$dir/portal.crt" ]; then
  openssl req -newkey rsa:4096 -nodes -x509 -days 3650 -sha256 \
    -keyout "$dir/portal.key" -out "$dir/portal.crt" \
    -subj "/CN=portail.localhost" \
    -addext "subjectAltName=DNS:portail.localhost,DNS:*.localhost,IP:127.0.0.1" 2>/dev/null
fi

chmod 644 "$dir"/*

[ -e "$here/.env" ] || printf 'LDAP_ADMIN_PASSWORD=%s\n' "$(cat "$dir/ldap-admin-pw")" >"$here/.env"

echo "secrets OK: $dir (+ $here/.env) — ne pas committer"
