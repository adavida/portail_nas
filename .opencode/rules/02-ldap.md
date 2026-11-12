# 02 — Annuaire — backends `ldap` | `sqlite`

> Sélection runtime par `config.toml`: `[directory] backend = "ldap" | "sqlite"` (règle 06). Les controllers/http/oidc ne voient que `repository::<fn>` qui dispatche (règle 03).

## LDAP — deux bases isolées

- `dev` (manuel, `devenv up`): `[ldap] url=ldap://127.0.0.1:3890`, base `dc=dev,dc=example,dc=com` — jamais touché par les tests.
- `test` (auto/CI): `config.test.toml` (backend sqlite) et `config.test.ldap.toml` (backend ldap, db de test séparée `db-test-ldap/`) → `[ldap] url=ldap://127.0.0.1:3891`, base `dc=test,dc=example,dc=com` — seul chargé par `cargo test` (via `set_test_config_*()`, règle 06). Les suites d'intégration tournent dans les DEUX binaires par backend (règle 03/10); backend injoignable ⇒ `assert_available()` met le test en échec (la suite complète nécessite `devenv up`). Les unit tests du repo ldap (`repository/ldap/**`) gardent leurs gardes de skip existantes.
- Pièges slapd: binaire `${pkgs.openldap}/libexec/slapd` (pas `bin/`), schémas `${pkgs.openldap}/etc/schema/*.schema`, `modulepath ${pkgs.openldap}/lib/modules` + `moduleload back_mdb.la` obligatoires dans `slapd.conf` sinon `daemon_init` échoue.
- Seed (si `data.mdb` absent): `slapadd -f slapd.conf -l /tmp/seed.ldif` — base `dcObject`+`organization`, `ou=people`, `ou=groups` (cf. `devenv.nix:84`, `devenv.nix:124`).
- Vérif: `devenv shell -- ldapsearch -x -H ldap://127.0.0.1:3890 -b "dc=dev,dc=example,dc=com" -D "cn=admin,dc=dev,dc=example,dc=com" -w admin` → `result: 0 Success`.

## SQLite

- Tables `users`/`groups`/`group_members` en migration `0002` (`backend/db`, règle 15) — mots de passe argon2 hashés dans `directory`, jamais en clair.
- Membres de groupe = `Uid` uniquement (plus de `cn=` libres, VO `Cn` supprimé).
- db de test: `db_path` de `config.test.toml`, purgée par `#[ctor]`/`#[dtor]` (`tests/common/db/purge.rs`, règle 10).
- Bootstrap `admin`: `main.rs` appelle `controllers::users::ensure_admin` au démarrage — no-op si le groupe `admin` a déjà au moins un membre ; sinon crée `admin` (mot de passe `directory.admin_password_file`, défaut `admin`) et l'ajoute au groupe `admin`; sur les deux backends. `delete_user` retire l'utilisateur de tous ses groupes (cascade SQLite et LDAP ; un groupe LDAP vidé est supprimé, `groupOfNames` exigeant ≥ 1 membre).
