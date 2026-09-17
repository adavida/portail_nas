# 02 — LDAP — Deux bases isolées

## Deux DN distincts

- `dev` (manuel, `devenv up`): `LDAP_URL=ldap://127.0.0.1:3890` / `LDAP_BASE_DN=dc=dev,dc=example,dc=com` → `processes.openldap` (3890)
- `test` (auto, CI): `LDAP_TEST_URL=ldap://127.0.0.1:3891` / `LDAP_TEST_BASE_DN=dc=test,dc=example,dc=com` → `processes."openldap-test"` (3891)
- Ne jamais polluer `dev` depuis les tests. `cargo test` utilise `LDAP_TEST_*` uniquement.

## Pièges Nix (à ne pas rater)

- Binaire: `${pkgs.openldap}/libexec/slapd` (pas `bin/slapd` — `bin` ne contient que `ldap*` + `slap*` symlinks vers `libexec/slapd`).
- Schémas: `${pkgs.openldap}/etc/schema/*.schema` (pas `etc/openldap/schema`).
- Obligatoire dans `slapd.conf`: `modulepath ${pkgs.openldap}/lib/modules` + `moduleload back_mdb.la` sinon `daemon_init` échoue.
- Vérif: `ls /nix/store/*openldap*/etc/schema/core.schema` et `ls /nix/store/*openldap*/libexec/slapd`.

## Seed

- Créer `slapd.conf` puis seed uniquement si `data.mdb` absent: `${pkgs.openldap}/bin/slapadd -f slapd.conf -l /tmp/seed.ldif`
- Seed LDIF: `dcObject`+`organization` pour base, `ou=people` + `ou=groups` (`organizationalUnit`). Cf. `devenv.nix:84` et `devenv.nix:124`.
- Test manuel: `devenv shell -- ldapsearch -x -H ldap://127.0.0.1:3890 -b "dc=dev,dc=example,dc=com" -D "cn=admin,dc=dev,dc=example,dc=com" -w admin` → `result: 0 Success`.
