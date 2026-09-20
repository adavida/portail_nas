# 03 — Backend & Frontend

> Skills: `backend-format` (trigger `*.rs`), `backend-structure` (trigger `http/`, `controllers/`, `domain/`), `frontend-format` + `frontend-structure` (trigger `frontend/src`, `*.tsx`).

## Backend `portail-backend` (Axum 0.7, Tokio full)

- Workspace `Cargo.toml:2` `members = ["backend"]`, crate `backend/Cargo.toml:2`.

## Tests API — `backend/tests/`

- **1 endpoint = au moins 1 test d'intégration** — nouvelle route sans test = pas finie.
- 1 fichier = 1 ressource (`health.rs`, `users.rs`); nouvelle ressource ⇒ nouveau fichier `tests/<ressource>.rs`.
- Router via `portail_backend::app()` + `tower::ServiceExt::oneshot` (pas de vrai serveur); helper par fichier `send(method, uri, Option<json>) -> (StatusCode, Value)`.
- LDAP test uniquement (`LDAP_TEST_*`, 3891) — jamais `dev` (3890).
- Auto-clean: `mod common;` + `tests/common/mod.rs` (`#[ctor] fn purge_test_ldap`, dev-dep `ctor`) purge `ou=people` de la base test au démarrage; LDAP down ⇒ no-op silencieux.
- Données uniques: uid timestamp (`TestUser::new`) — prefix `^[a-z0-9._-]+$` (pénètre dans l'URL). Pas de cleanup par test — la purge `ctor` suffit (sauf tests dont le DELETE est le sujet).
- Corps 204 vide ⇒ parse JSON avec fallback `unwrap_or(json!({}))`. LDAP inatteignable ⇒ skip via guard (`500`/`Err(AppError::Ldap)`), CI sans LDAP reste verte.
- **Routes test-only** (ex: `POST /api/users/:uid/authenticate`): feature flag `test-api` (`backend/Cargo.toml` `[features]`, `#[cfg(feature = "test-api")]` dans `users_router`) — absentes du build prod. **Piège**: `#[cfg(test)]` est ignoré dans la lib quand `backend/tests/*.rs` la compilent comme dépendance — dans `tests/`, seul le feature flag filtre (`#[cfg(feature = "test-api")]` au-dessus de `#[tokio::test]`); `#[cfg(test)]` réservé aux unit tests collocalisés.
- Verif: `devenv shell -- cargo test --features test-api -- --nocapture` (unit + intégration).

## Frontend `portail-frontend` (Vite + React 18 + TS)

- `package.json:6` scripts `dev`/`build`/`test` (`vitest run`)/`test:watch`. `vite.config.ts:12` jsdom + `setupFiles=./src/test/setup.ts`.
- `vite.config.ts:8` proxy `/api → http://localhost:3000`, port `5173`; `pages/Users.tsx` fetch `/api/users`, `App.tsx` glue.
- Tests collocalisés `@testing-library/react` + `jsdom` (`*.test.tsx` collocalisés, `App.test.tsx`).
- Commandes: `devenv shell -- npm --prefix frontend test -- --run` (single: `-t "renders title"`); prérequis `devenv shell -- bash -c 'cd frontend && npm install'`.

## Lien backend ↔ frontend

- `vite.config.ts:8` proxy doit matcher `main.rs` `bind 0.0.0.0:3000`. Changer l'un ⇒ changer l'autre + `justfile:5`.
