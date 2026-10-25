# 04 — Workflow, TDD, VSCode, Sécurité, Layout

## Environnement & process auto

- Toujours `devenv shell -- <cmd>` (rust 1.98, nodejs_22). `devenv up` lance 8 process: `backend` (génère `config.toml` dans `$PORTAIL_CONFIG_FILE` (heredoc depuis le bloc `env`, clients OIDC inline dans `oidc_clients`: portail activé, `oauth-debugger` désactivé) puis `cargo watch -w backend -x 'run -p portail-server -- --config config.toml'` → 3000, fournisseur OIDC inclus, issuer http://127.0.0.1:3000), `backend-test` (`cargo watch -w backend -x test`), `frontend` (`vite` 5173), `frontend-test` (`vitest` watch), `oauthdebugger` (`uvx oauth2-debugger` → 3030, redirect_uri `http://127.0.0.1:3030/callback`, client `oauth-debugger` pré-déclaré désactivé dans `oidc_clients` — l'activer dans le config généré + restart), `openldap` (3890), `openldap-test` (3891), `vscode`. `devenv down` arrête. Ne pas lancer `cargo run`/`npm run dev` hors process.
- Le bloc `env` de `devenv.nix` (`LDAP_*`, `OIDC_*`, `APP_URL`, `BIND_ADDR`, `LDAP_TEST_*`) sert uniquement aux **scripts shell** (seed slapd, génération config.toml, resetUsers, enterShell) — le binaire backend ne lit **aucune** de ces vars (règle 06: `--config` + TOML).
- Après modif `devenv.nix`/`devenv.yaml`/`opencode.json`/`.opencode/**` → quitter et relancer opencode.

## TDD

- Watchers `backend-test`/`frontend-test` tournent en permanence — ne pas relancer `cargo test` en boucle manuelle.
- `just test` (`justfile:9`) = `cargo test` + `npm --prefix frontend test -- --run` (raccourci local).
- Nouvelle route LDAP → trait `LdapClient` mockable, pas de vrai LDAP en unit test (intégration seulement via `config.test.toml`).
- Tests AAA : tout `#[test]` / `test(` suit Arrange, Act, Assert avec **ligne vide entre les 3** + **assertions faciles à lire** (`assert_eq!` avec message, variable intermédiaire, `assert_auth_*` helpers) — skill `.opencode/skills/test-aaa/SKILL.md` trigger `*.rs` `*.test.ts`.
- **1 fonction publique = au moins 1 test** : toute `pub fn`/`pub async fn` (domain, repository, controllers, http) a un test collocalisé `#[cfg(test)]` (repo/domain) ou d'intégration `backend/server/tests/*.rs` (routes). Une fonction publique sans test = pas finie.
- Après toute modif Rust (`backend/*/src/**/*.rs`, `Cargo.toml`) → `devenv shell -- cargo fmt` obligatoire en fin de modif, puis `devenv shell -- treefmt --fail-on-change` avant `git add` — skill `.opencode/skills/backend-format/SKILL.md`.

## VSCode

- `codiumWithExt = pkgs.vscode-with-extensions.override { vscode = pkgs.vscodium; vscodeExtensions = [rust-analyzer even-better-toml vscode-lldb prettier-vscode vscode-eslint nix-ide direnv vscode-tailwindcss svelte-vscode] }` dans `packages` et `processes.vscode.exec = "${codiumWithExt}/bin/codium .; sleep infinity"`. Extensions aussi dans `.vscode/extensions.json`/`settings.json` — ne pas installer via Marketplace.

## Sécurité & garde-fous

- Ne jamais committer `.devenv/`, `target/`, `node_modules/`, `frontend/dist/` (`.gitignore:1`).
- Ne pas exposer `cn=admin`/`admin` hors `devenv.nix` local. Pas de secrets en dur dans `backend/*/src`.
- `allow_unfree: true` dans `devenv.yaml` volontaire (VSCodium).
- Permissions opencode (`opencode.json`): `bash` allowlist `devenv/cargo/npm/just/git/ldap*/treefmt` → `allow`, reste `ask`; `external_directory` locké au repo.

## Layout

- Racine: `devenv.nix` / `devenv.yaml` / `devenv.lock` / `.envrc` (`/home/david/projects/rust/nas/202260917_portail`)
- Code: `backend/{core,directory,oidc,server}/src/`, `backend/server/src/main.rs` (bin `portail-backend`), `backend/server/src/http/mod.rs` (`router()`), `frontend/src/` (`App.svelte`, `main.ts`, `App.test.ts`, `vite.config.ts`)
- Config: `.vscode/{settings,extensions}.json`, `justfile`, `.opencode/rules/01-devenv.md` … `04-workflow.md`
