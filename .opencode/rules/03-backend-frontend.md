# 03 — Backend & Frontend

> Skills de référence:
>
> - Formatage/lints: `.opencode/skills/backend-format/SKILL.md` (trigger `backend/src`, `Cargo.toml`, `*.rs`)
> - Structure/serde: `.opencode/skills/backend-structure/SKILL.md` (trigger `backend/src/lib.rs`, `http/`, `controllers/`, `domain/`)

## Backend `portail-backend` (Axum 0.7, Tokio full)

- Workspace `Cargo.toml:2` `members = ["backend"]`, crate `backend/Cargo.toml:2`. Voir skill ci-dessus pour `rustfmt`/`clippy`/`serde`/`domain`/`http`/`controllers`.

## Tests API — `backend/tests/`

- **1 endpoint = au moins 1 test d'intégration** dans `backend/tests/` — une nouvelle route sans test d'intégration n'est pas finie.
- Routes **test-only** (ex: `POST /api/users/:uid/authenticate`) derrière le feature flag `test-api` (`backend/Cargo.toml` `[features]`, gate `#[cfg(feature = "test-api")]` dans `users_router`) — absentes du build `cargo run`/prod. Les tests/`justfile`/`devenv.nix` backends invoquent `cargo test --features test-api`. **Piège**: `#[cfg(test)]` est ignoré dans le build de la lib quand les binaires `backend/tests/*.rs` la compilent comme dépendance normale — dans `tests/`, seul le feature flag filtre (et sur un test de route test-only: `#[cfg(feature = "test-api")]` au-dessus de `#[tokio::test]`); `#[cfg(test)]` reste réservé aux unit tests collocalisés (`mod tests` dans les fichiers sources).
- 1 fichier = 1 ressource: `health.rs` (GET `/api/health` + 404), `users.rs` (GET/POST/PUT/DELETE `/api/users...`). Nouvelle ressource ⇒ nouveau fichier `tests/<ressource>.rs`.
- Router via `portail_backend::app()` + `tower::ServiceExt::oneshot` (pas de vrai serveur HTTP). Helpers par fichier: `send(method, uri, Option<json>) -> (StatusCode, Value)`.
- LDAP test uniquement (`LDAP_TEST_*`, 3891) via les process `devenv up` — jamais `dev` (3890).
- Auto-clean: chaque binaire de test purge `ou=people` de la base **test** avant de rouler — `mod common;` en tête de fichier inclut `tests/common/mod.rs` (`#[ctor] fn purge_test_ldap`, dev-dep `ctor`). LDAP down ⇒ purge silencieuse (no-op).
- Données de test uniques: uid généré timestamp (`TestUser::new`) — prefix uid **sans espace/chiffres interdits** (`^[a-z0-9._-]+$`, le uid pénètre dans l'URL). Cleanup par test supprimé — la purge `ctor` est la seule garantie (exepté les tests dont le DELETE est le sujet du test).
- Corps 204 vide ⇒ parse JSON avec fallback `unwrap_or(json!({}))`.
- Si LDAP inatteignable: skip via guard (`500`/`Err(AppError::Ldap)`) au lieu de paniquer — CI sans LDAP doit rester verte pour le code pur.
- Points deverification: `devenv shell -- cargo test --features test-api -- --nocapture` roule unit + intégration (toutes les routes).

## Frontend `portail-frontend` (Vite + React 18 + TS)

> Skills de référence:
>
> - Formatage/lints: `.opencode/skills/frontend-format/SKILL.md` (trigger `frontend/src`, `*.tsx`, `eslint.config.js`)
> - Structure: `.opencode/skills/frontend-structure/SKILL.md` (trigger `frontend/src/main.tsx`, `pages/`, `components/`)

- `frontend/package.json:6` scripts `dev`/`build`/`test` (`vitest run`)/`test:watch` (`vitest`). `frontend/vite.config.ts:12` `test.environment=jsdom`, `setupFiles=./src/test/setup.ts`.
- `frontend/vite.config.ts:8` proxy `/api → http://localhost:3000`, port `5173`. `frontend/src/pages/Users.tsx` fetch `/api/users`, `App.tsx` glue `pages/Users`.
- Commandes: `devenv shell -- npm --prefix frontend test -- --run`, single `devenv shell -- npm --prefix frontend test -- --run -t "renders title"`. Prérequis `devenv shell -- bash -c 'cd frontend && npm install'` (pas de `node` hors devenv).
- Tests collocalisés `@testing-library/react` + `jsdom` (`pages/Users.test.tsx`, `components/*.test.tsx`, `App.test.tsx`).

## Lien backend ↔ frontend

- `frontend/vite.config.ts:8` proxy doit matcher `backend/src/main.rs:22` `bind 0.0.0.0:3000`. Changer l'un ⇒ changer l'autre + `justfile:5`.
