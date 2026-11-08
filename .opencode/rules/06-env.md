# 06 — Config — Fichier TOML, zéro variable d'env backend

> Backend: `backend/core/src/config/` + `config.toml` / `config.test.toml` racine — Frontend: `frontend/src/env.ts` (Vite, build-time — hors périmètre env runtime).

## Objectif

Aucune variable d'env de configuration pour le backend. Le binaire se lance avec `portail-backend --config <chemin>` ; `Config::load(path)` (dans `backend/core/src/config/`) parse le TOML et construit le struct `Config`. Clap interdit — parsing argv à la main dans `main.rs` (5 lignes), `panic!` avec message `usage: ...` si `--config` absent.

## Structure

- `config/mod.rs`: `struct Config` (sous-structs `LdapConfig`/`OidcConfig`/`SecurityConfig` + champs plats `app_url`/`bind_addr`/`db_path`/`system_sensors`) + `RawConfig` serde (`deny_unknown_fields`, sections TOML `[ldap]`/`[oidc]`/`[security]`) + `load(path)` + `log_summary()` + défauts `DEFAULT_*` (`DEFAULT_KEY_BITS`, `DEFAULT_PENDING_TTL_SECS`, `DEFAULT_CODE_TTL_SECS`, `DEFAULT_SESSION_TTL_SECS`).
- `config/global.rs`: `static CONFIG: OnceLock<Config>` + `set()` (2e appel ignoré) + `global()` + `ensure_init(path)` (idempotent).
- `config/errors.rs`: `ConfigError`.
- Clés requises (absentes = erreur serde, fail-fast): `app_url`, `bind_addr`, `ldap.url`, `ldap.base_dn`, `oidc.issuer_url`, `oidc.client_id`, `oidc.redirect_uri`, `oidc.secrets_dir`.
- Clés optionnelles avec défaut: `ldap.people_ou`→people, `ldap.groups_ou`→groups, `ldap.tls_ca`→vide, `ldap.admin_pw_file`→absent = `admin` (valeur lue depuis le fichier, trim), `oidc.key_bits`→2048, `oidc.secret_logs`→true, `oidc.client_owners`→vide, `oidc.clients`→[], `oidc.*_ttl_secs`→300/60/43200/3600, `oidc.id_bytes`→16 (plancher validé au load, jamais sous 16), `[security]` absent = défauts, `security.password_min_length`→8, `security.login_max_attempts`→10.
- chemins relatifs dans le TOML (`oidc.secrets_dir`, `ldap.tls_ca`, `ldap.admin_pw_file`) résolus contre le répertoire du fichier de config (jamais le cwd).
- Fichier illisible / TOML invalide / champ manquant ou inconnu / secret file vide = `ConfigError` → `main` panique avec le message complet.

## Fichiers

- `config.toml` (racine, commité): valeurs dev (LDAP 3890 `dc=dev`, chemins `.devenv/state/...` relatifs). Lancé par `devenv up` via `--config config.toml`.
- `config.test.toml` (racine, commité): LDAP 3891 `dc=test`, secrets `.devenv/state/secrets-test`. Chargé par les helpers de test via `env!("CARGO_MANIFEST_DIR")/../../config.test.toml` (constante de build, pas d'env runtime).

## Tests

- Aucun `std::env::var` dans les tests : `set_test_config()` (1 helper = 1 fichier, `src/test_helpers/config/` et `tests/common/config/`) charge `config.test.toml` via `Config::ensure_init` — `#[ctor]` dans les crates src, appel explicite dans `tests/`.
- LDAP test uniquement (3891, `dc=test`) — jamais `dev` (règle 02).

## Interdits

- `std::env::var` / `option_env!` / `env!` hors `backend/core/src/config/**` et hors `#[cfg(test)]` (seule exception: `env!("CARGO_MANIFEST_DIR")` dans les helpers de test, `std::env::args` dans `main.rs`).
- Toute nouvelle clé de config passe par `RawConfig` — jamais de lecture directe ailleurs.
- Frontend: `import.meta.env` reste cantonné à `frontend/src/env.ts` (inchangé).

## Vérification

```bash
rg -n "std::env::var|option_env!" backend/*/src backend/server/tests --glob '*.rs' # vide hors config/** et cfg(test)
rg -n 'env!\("' backend --glob '*.rs' -g '!target' # uniquement CARGO_MANIFEST_DIR
devenv shell -- cargo test --features test-api
```
