# 02 — LDAP — Deux bases isolées

- `dev` (manuel, `devenv up`): `LDAP_URL=ldap://127.0.0.1:3890`, base `dc=dev,dc=example,dc=com` — jamais touché par les tests.
- `test` (auto/CI): `LDAP_TEST_URL=ldap://127.0.0.1:3891`, base `dc=test,dc=example,dc=com` — seul utilisé par `cargo test` (`LDAP_TEST_*`).
- Pièges slapd: binaire `${pkgs.openldap}/libexec/slapd` (pas `bin/`), schémas `${pkgs.openldap}/etc/schema/*.schema`, `modulepath ${pkgs.openldap}/lib/modules` + `moduleload back_mdb.la` obligatoires dans `slapd.conf` sinon `daemon_init` échoue.
- Seed (si `data.mdb` absent): `slapadd -f slapd.conf -l /tmp/seed.ldif` — base `dcObject`+`organization`, `ou=people`, `ou=groups` (cf. `devenv.nix:84`, `devenv.nix:124`).
- Vérif: `devenv shell -- ldapsearch -x -H ldap://127.0.0.1:3890 -b "dc=dev,dc=example,dc=com" -D "cn=admin,dc=dev,dc=example,dc=com" -w admin` → `result: 0 Success`.
