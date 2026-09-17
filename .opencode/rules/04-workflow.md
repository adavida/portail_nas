# 04 — Workflow, TDD, VSCode, Sécurité, Layout

## Environnement & process auto

- Toujours `devenv shell -- <cmd>` (rust 1.98, nodejs_22). `devenv up` lance 7 process: `backend` (`cargo watch -w backend -x 'run -p portail-backend'` → 3000), `backend-test` (`cargo watch -w backend -x test`), `frontend` (`vite` 5173), `frontend-test` (`vitest` watch), `openldap` (3890), `openldap-test` (3891), `vscode`. `devenv down` arrête. Ne pas lancer `cargo run`/`npm run dev` hors process.
- Après modif `devenv.nix`/`devenv.yaml`/`opencode.json`/`.opencode/**` → quitter et relancer opencode.

## TDD

- Watchers `backend-test`/`frontend-test` tournent en permanence — ne pas relancer `cargo test` en boucle manuelle.
- `just test` (`justfile:9`) = `cargo test` + `npm --prefix frontend test -- --run` (raccourci local).
- Nouvelle route LDAP → trait `LdapClient` mockable, pas de vrai LDAP en unit test (intégration seulement via `LDAP_TEST_*`).
- Tests AAA : tout `#[test]` / `test(` suit Arrange, Act, Assert avec **ligne vide entre les 3** + **assertions faciles à lire** (`assert_eq!` avec message, variable intermédiaire, `assert_auth_*` helpers) — skill `.opencode/skills/test-aaa/SKILL.md` trigger `*.rs` `*.test.tsx`.

## VSCode

- `codiumWithExt = pkgs.vscode-with-extensions.override { vscode = pkgs.vscodium; vscodeExtensions = [rust-analyzer even-better-toml vscode-lldb prettier-vscode vscode-eslint nix-ide direnv vscode-tailwindcss] }` dans `packages` et `processes.vscode.exec = "${codiumWithExt}/bin/codium .; sleep infinity"`. Extensions aussi dans `.vscode/extensions.json`/`settings.json` — ne pas installer via Marketplace.

## Sécurité & garde-fous

- Ne jamais committer `.devenv/`, `target/`, `node_modules/`, `frontend/dist/` (`.gitignore:1`).
- Ne pas exposer `cn=admin`/`admin` hors `devenv.nix` local. Pas de secrets en dur dans `backend/src`.
- `allow_unfree: true` dans `devenv.yaml` volontaire (VSCodium).
- Permissions opencode (`opencode.json`): `bash` allowlist `devenv/cargo/npm/just/git/ldap*/treefmt` → `allow`, reste `ask`; `external_directory` locké au repo.

## Layout

- Racine: `devenv.nix` / `devenv.yaml` / `devenv.lock` / `.envrc` (`/home/david/projects/rust/nas/202260917_portail`)
- Code: `backend/src/main.rs`, `backend/src/{domain,http,controllers}/`, `frontend/src/` (`App.tsx`, `main.tsx`, `App.test.tsx`, `vite.config.ts`)
- Config: `.vscode/{settings,extensions}.json`, `justfile`, `.opencode/rules/01-devenv.md` … `04-workflow.md`
