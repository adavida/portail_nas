#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: migrate-authelia-subs.sh <mapping.json> <authelia.db> <portail.db> [--dry-run]

Imports Authelia openid pairwise subject identifiers into the portail database
and prints the matching oidc_clients TOML block for config.toml on stdout.

mapping.json format:
  { "clients": [
      { "sector_id": "app.example.com", "id": "myapp",
        "secret_file": "myapp_secret", "enabled": true,
        "redirect_uris": ["https://app.example.com/callback"],
        "scopes": ["openid", "profile", "email"] }
  ] }

authelia.db is never modified (a temporary copy is read).
INSERT OR IGNORE only: existing portail subjects are never overwritten.
Client secrets are hashed in Authelia and cannot be imported: regenerate them
in the portail secrets dir and push the new values to the applications.
EOF
  exit 1
}

print_oidc_clients_toml() {
  jq -r '
    .clients
    | "oidc_clients = [\n"
    + (map(
        "  { id = " + (.id | tojson)
        + (if .secret_file then ", secret_file = " + (.secret_file | tojson) else "" end)
        + ", enabled = " + ((.enabled // false) | tostring)
        + ", redirect_uris = [" + (.redirect_uris | map(tojson) | join(", ")) + "]"
        + ", scopes = [" + (.scopes // [] | map(tojson) | join(", ")) + "] }"
      )
      | join(",\n"))
    + "\n]"' "$MAPPING"
}

build_inserts_sql() {
  jq -r --arg q "'" '
    def esc: gsub($q; $q + $q);
    .[] |
    "INSERT OR IGNORE INTO user_opaque_identifier (client_id, username, identifier) VALUES ("
    + $q + (.client_id | esc) + $q + ", "
    + $q + (.username | esc) + $q + ", "
    + $q + (.identifier | esc) + $q + ");"' "$MAPPED_TMP"
}

[ $# -ge 3 ] && [ $# -le 4 ] || usage
MAPPING="$1"
AUTHELIA_DB="$2"
PORTAIL_DB="$3"
DRY_RUN=false
if [ "$#" -eq 4 ]; then
  [ "$4" = "--dry-run" ] || usage
  DRY_RUN=true
fi

for bin in sqlite3 jq pgrep; do
  command -v "$bin" >/dev/null || { echo "error: $bin is required" >&2; exit 1; }
done
[ -f "$MAPPING" ] || { echo "error: missing mapping file $MAPPING" >&2; exit 1; }
[ -f "$AUTHELIA_DB" ] || { echo "error: missing authelia db $AUTHELIA_DB" >&2; exit 1; }

jq -e '.clients | (type == "array") and (length > 0)' "$MAPPING" >/dev/null ||
  { echo "error: mapping must contain a non-empty clients array" >&2; exit 1; }

if pgrep -f portail-backend >/dev/null 2>&1; then
  echo "error: portail-backend is running, stop it before importing" >&2
  exit 1
fi

AUTHELIA_TMP="$(mktemp)"
ROWS_TMP="$(mktemp)"
MAPPED_TMP="$(mktemp)"
trap 'rm -f "$AUTHELIA_TMP" "$ROWS_TMP" "$MAPPED_TMP"' EXIT
cp "$AUTHELIA_DB" "$AUTHELIA_TMP"

sqlite3 "$AUTHELIA_TMP" \
  "SELECT count(*) FROM user_opaque_identifier WHERE service = 'openid';" >/dev/null 2>&1 ||
  { echo "error: $AUTHELIA_DB has no user_opaque_identifier table" >&2; exit 1; }

sqlite3 -json "$AUTHELIA_TMP" "
SELECT sector_id, username,
  CASE typeof(identifier)
    WHEN 'blob' THEN substr(hex(identifier),1,8)||'-'||substr(hex(identifier),9,4)||'-'||substr(hex(identifier),13,4)||'-'||substr(hex(identifier),17,4)||'-'||substr(hex(identifier),21,12)
    ELSE lower(identifier)
  END AS identifier
FROM user_opaque_identifier
WHERE service = 'openid';" >"$ROWS_TMP"

[ -s "$ROWS_TMP" ] || { echo "error: no openid rows in $AUTHELIA_DB" >&2; exit 1; }

jq -n --slurpfile mapping "$MAPPING" --slurpfile rows "$ROWS_TMP" '
  ($mapping[0].clients | map({key: .sector_id, value: .id}) | from_entries) as $sector_to_client
  | ($rows[0]
      | map(select($sector_to_client[.sector_id] == null))
      | map(.sector_id) | unique) as $orphans
  | if ($orphans | length) > 0 then
      error("unmapped sector_id in authelia db: " + ($orphans | join(", ")))
    else
      $rows[0] | map({
        client_id: $sector_to_client[.sector_id],
        username: .username,
        identifier: .identifier
      })
    end' >"$MAPPED_TMP"

if [ "$DRY_RUN" = true ]; then
  echo "dry run: would import the following into $PORTAIL_DB" >&2
  build_inserts_sql >&2
  print_oidc_clients_toml
  exit 0
fi

[ -f "$PORTAIL_DB" ] || touch "$PORTAIL_DB"
chmod 600 "$PORTAIL_DB"
sqlite3 "$PORTAIL_DB" "CREATE TABLE IF NOT EXISTS user_opaque_identifier (
  client_id TEXT NOT NULL,
  username TEXT NOT NULL,
  identifier TEXT NOT NULL,
  PRIMARY KEY (client_id, username)
);"

BEFORE=$(sqlite3 "$PORTAIL_DB" "SELECT count(*) FROM user_opaque_identifier;")
sqlite3 "$PORTAIL_DB" "$(build_inserts_sql)"
AFTER=$(sqlite3 "$PORTAIL_DB" "SELECT count(*) FROM user_opaque_identifier;")

TOTAL=$(jq 'length' "$MAPPED_TMP")
echo "imported: $((AFTER - BEFORE)) subs, skipped (already present): $((TOTAL - (AFTER - BEFORE))), total authelia rows: $TOTAL" >&2

print_oidc_clients_toml
