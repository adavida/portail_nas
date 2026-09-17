# AGENTS.md

## Env - devenv is source of truth

- Always run via `devenv shell -- <cmd>` (rust 1.98, nodejs_22, openldap, cargo-watch, treefmt provided). Bare `cargo`/`node` may be missing.
- `direnv allow` or `devenv shell` to enter. `devenv.yaml` has `allow_unfree: true` for codium.
- `devenv up` starts all processes, `deven down` stops. `treefmt` runs automatically on `enterShell` (formats 15 files, 3 changed typical).
- `devenv.nix` conventions (enforced by `nixfmt`): top-level attrs alphabetical (`env` < `enterShell` < `enterTest` < `languages` < `packages` < `processes` < `scripts` < `tasks` < `treefmt`); group when >1 prop `env = { LDAP_* }`, `languages.javascript = { }`; singletons stay dotted `scripts.hello.exec`, `languages.rust.enable`, `processes.openldap.exec`. No comments.

## Services & env

- Two isolated LDAPs (same slapd config, different DBs):
  - dev: `LDAP_URL=ldap://127.0.0.1:3890` `LDAP_BASE_DN=dc=dev,dc=example,dc=com` → `processes.openldap` (port 3890)
  - test: `LDAP_TEST_URL=ldap://127.0.0.1:3891` `LDAP_TEST_BASE_DN=dc=test,dc=example,dc=com` → `processes."openldap-test"` (port 3891)
- LDAP gotcha: binary is `${pkgs.openldap}/libexec/slapd` (not `bin/slapd`), schemas at `${pkgs.openldap}/etc/schema/*.schema` (not `etc/openldap/schema`), need `modulepath .../lib/modules` + `moduleload back_mdb.la`. Seed only when `data.mdb` missing via `slapadd -f slapd.conf -l /tmp/seed*.ldif` (ou=people, ou=groups).
- VSCode: `codiumWithExt = pkgs.vscode-with-extensions.override { vscode = pkgs.vscodium; vscodeExtensions = [rust-analyzer even-better-toml vscode-lldb prettier-vscode vscode-eslint nix-ide direnv vscode-tailwindcss] }` in `packages` and `processes.vscode.exec = "${codiumWithExt}/bin/codium .; sleep infinity"`. Extensions also in `.vscode/extensions.json`/`settings.json` — don't install manually.

## Processes (`devenv up`)

- `backend` `cargo watch -w backend -x 'run -p portail-backend'` → `0.0.0.0:3000`
- `backend-test` `cargo watch -w backend -x test`
- `frontend` `npm --prefix frontend run dev` → Vite `5173` proxy `/api` → `3000` (`frontend/vite.config.ts:8`)
- `frontend-test` `npm --prefix frontend run test:watch` (vitest)
- `openldap` / `openldap-test` (see above) + `vscode`
- Alphabetical inside `processes`: `backend` < `backend-test` < `frontend` < `frontend-test` < `openldap` < `openldap-test` < `vscode`.

## Backend `portail-backend` (Axum 0.7)

- Workspace `Cargo.toml:2` members `["backend"]`, crate `backend/Cargo.toml:2`.
- Entrypoint `backend/src/main.rs:15` `Router::new().route("/api/health", get(health))`. Tests `backend/src/main.rs:27` `#[tokio::test]` using `tower::ServiceExt::oneshot` + `http_body_util::BodyExt`.
- Commands: `devenv shell -- cargo test` (or `tasks "portail:backend:test"`), single test `devenv shell -- cargo test health_returns_ok -- --nocapture`.

## Frontend `portail-frontend` (Vite+React+TS)

- `frontend/package.json:6` scripts `dev`/`build`/`test` (`vitest run`)/`test:watch` (`vitest`). `frontend/vite.config.ts:12` `test.environment=jsdom`.
- `frontend/src/App.tsx:7` fetches `/api/health`. Tests `frontend/src/App.test.tsx` with `vitest` + `jsdom` + `@testing-library/react`.
- Commands: `devenv shell -- npm --prefix frontend test -- --run`, single `devenv shell -- npm --prefix frontend test -- --run -t "renders title"`. Need `devenv shell -- bash -c 'cd frontend && npm install'` first (no `node` outside devenv).

## Formatting & tasks

- `treefmt` (`devenv.nix:154`): `enable = true`, `config.projectRootFile = "devenv.nix"`, `programs.nixfmt/rustfmt/prettier.enable`. Input `treefmt-nix` in `devenv.yaml:5` (follows nixpkgs). Run `devenv shell -- treefmt` or let `enterShell` run it. Don't call `rustfmt`/`nixfmt` directly.
- `justfile:9` `just test` = `cargo test` + `npm --prefix frontend test -- --run` (legacy, prefer `devenv shell` tasks).

## Layout

- `devenv.nix` / `devenv.yaml` / `devenv.lock` at repo root (`/home/david/projects/rust/nas/202260917_portail`)
- `backend/src/main.rs`, `frontend/src/` (`App.tsx`, `main.tsx`, `App.test.tsx`, `vite.config.ts`)
- `.vscode/{settings,extensions}.json`, `.gitignore` excludes `.devenv/`, `target/`, `node_modules/`, `frontend/dist/`
