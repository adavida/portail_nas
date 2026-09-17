# 03 — Backend & Frontend

## Backend `portail-backend` (Axum 0.7, Tokio full)

- Workspace `Cargo.toml:2` `members = ["backend"]`, crate `backend/Cargo.toml:2`.
- Entrypoint `backend/src/main.rs:15` `Router::new().route("/api/health", get(health))` → `{"status":"ok"}`. Ne pas déplacer sans MAJ `frontend/src/App.tsx:7` et `frontend/vite.config.ts:8` proxy.
- Tests collocalisés `#[tokio::test]` avec `tower::ServiceExt::oneshot` + `http_body_util::BodyExt` (`backend/src/main.rs:27`).
- Commandes: `devenv shell -- cargo test` (ou `tasks "portail:backend:test"`), single `devenv shell -- cargo test health_returns_ok -- --nocapture`.

## Frontend `portail-frontend` (Vite + React 18 + TS)

- `frontend/package.json:6` scripts `dev`/`build`/`test` (`vitest run`)/`test:watch` (`vitest`). `frontend/vite.config.ts:12` `test.environment=jsdom`, `setupFiles=./src/test/setup.ts`.
- `frontend/vite.config.ts:8` proxy `/api → http://localhost:3000`, port `5173`. `frontend/src/App.tsx:7` fetch `/api/health`.
- Commandes: `devenv shell -- npm --prefix frontend test -- --run`, single `devenv shell -- npm --prefix frontend test -- --run -t "renders title"`. Prérequis `devenv shell -- bash -c 'cd frontend && npm install'` (pas de `node` hors devenv).
- Tests `@testing-library/react` + `jsdom` (`frontend/src/App.test.tsx`). Fichiers `*.test.tsx` à côté du code.

## Lien backend ↔ frontend

- `frontend/vite.config.ts:8` proxy doit matcher `backend/src/main.rs:22` `bind 0.0.0.0:3000`. Changer l'un ⇒ changer l'autre + `justfile:5`.
