# 03 — Backend & Frontend

> Skills: `backend-format` (trigger `*.rs`), `backend-structure` (trigger `http/`, `controllers/`, `domain/`), `frontend-format` + `frontend-structure` (trigger `frontend/src`, `*.svelte`).

## Backend workspace — 4 crates (Axum 0.8, Tokio full)

- Workspace `Cargo.toml` `members = ["backend/core", "backend/directory", "backend/oidc", "backend/server"]`.
- `portail-core` (`backend/core`): `config/` (TOML, `Config`/`ConfigError`) + `error` (AppError + `IntoResponse`) — dépendance de tous.
- `portail-directory` (`backend/directory`): `domain/{users,groups}`, `repository/ldap`, `controllers/{users,groups}`, `http/{users,groups}` — routers **sans middleware**, `ldap3`/`native-tls` isolés ici.
- `portail-oidc` (`backend/oidc`): `auth/` (dont middleware `require_auth`/`require_admin`), `domain/oidc`, `controllers/oidc`, `http/{oidc,auth}` — dépend directory (login authentifie via LDAP).
- `portail-server` (`backend/server`): assemblage `router()` (`lib.rs` `pub use http::router as app`), health, `main.rs` (bin `portail-backend` via `[[bin]]`), `tests/` intégration — dépend les trois autres.
- Graphe acyclique imposée: server → oidc → directory → core. Aucun crate métier ne câble de middleware; l'assemblage est seul dans server.

## Tests API — `backend/server/tests/`

- **1 endpoint = au moins 1 test d'intégration** — nouvelle route sans test = pas finie.
- 1 fichier = 1 ressource (`health.rs`, `users.rs`); nouvelle ressource ⇒ nouveau fichier `tests/<ressource>.rs`.
- Router via `portail_server::app()` + `tower::ServiceExt::oneshot` (pas de vrai serveur); helper partagé `send(method, uri, Option<json>) -> (StatusCode, Value)` dans `tests/common/send.rs` (règle `10-test-helpers.md`).
- LDAP test uniquement (`config.test.toml`, 3891) — jamais `dev` (3890).
- Auto-clean: `mod common;` + `tests/common/purge.rs` (`#[ctor] fn purge_test_ldap`, dev-dep `ctor`) purge `ou=people` de la base test au démarrage; LDAP down ⇒ no-op silencieux.
- Données uniques: uid timestamp (`TestUser::new`) — prefix `^[a-z0-9._-]+$` (pénètre dans l'URL). Pas de cleanup par test — la purge `ctor` suffit (sauf tests dont le DELETE est le sujet).
- Corps 204 vide ⇒ parse JSON avec fallback `unwrap_or(json!({}))`. LDAP inatteignable ⇒ skip via guard (`500`/`Err(AppError::Ldap)`), CI sans LDAP reste verte.
- **Routes test-only** (ex: `POST /api/users/:uid/authenticate`): feature flag `test-api` déclaré dans `backend/directory/Cargo.toml` + `backend/oidc/Cargo.toml` `[features]`, relayé par `backend/server/Cargo.toml` (`test-api = ["portail-directory/test-api", "portail-oidc/test-api"]`), `#[cfg(feature = "test-api")]` dans `users_router`/`protected_auth_router` — absentes du build prod. **Piège**: `#[cfg(test)]` est ignoré dans la lib quand `backend/server/tests/*.rs` la compilent comme dépendance — dans `tests/`, seul le feature flag filtre (`#[cfg(feature = "test-api")]` au-dessus de `#[tokio::test]`); `#[cfg(test)]` réservé aux unit tests collocalisés.
- Verif: `devenv shell -- cargo test --features test-api -- --nocapture` (unit + intégration).

## Frontend `portail-frontend` (Vite + Svelte 5 runes + TS)

- `package.json:6` scripts `dev`/`build`/`test` (`vitest run`)/`test:watch`/`check` (`svelte-check`). `vite.config.ts` jsdom + `setupFiles=./src/test/setup.ts` + plugin `svelteTesting()` sous `VITEST`; preprocess dans `svelte.config.js` (`vitePreprocess`).
- proxy `/api → backend`, port `5173`; `pages/Users.svelte` fetch `/api/users`, `App.svelte` glue (pas de routeur: `window.location.pathname === "/callback"`).
- Tests collocalisés `@testing-library/svelte` + `jsdom` (`<X>.test.ts` à côté de `<X>.svelte`, `App.test.ts`); toast store resetté dans `test/setup.ts`.
- Commandes: `devenv shell -- npm --prefix frontend test -- --run` (single: `-t "renders title"`); prérequis `devenv shell -- bash -c 'cd frontend && npm install'`.

## Lien backend ↔ frontend

- Le proxy `vite.config.ts` doit matcher `main.rs` `bind 0.0.0.0:3000`. Changer l'un ⇒ changer l'autre + `justfile:5`.
