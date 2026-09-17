# 03 — Backend & Frontend

> Skills de référence:
>
> - Formatage/lints: `.opencode/skills/backend-format/SKILL.md` (trigger `backend/src`, `Cargo.toml`, `*.rs`)
> - Structure/serde: `.opencode/skills/backend-structure/SKILL.md` (trigger `backend/src/lib.rs`, `routes/`, `domain/`)

## Backend `portail-backend` (Axum 0.7, Tokio full)

- Workspace `Cargo.toml:2` `members = ["backend"]`, crate `backend/Cargo.toml:2`. Voir skills ci-dessus pour `rustfmt`/`clippy`/`serde`/`domain`/`routes`.

## Frontend `portail-frontend` (Vite + React 18 + TS)

- `frontend/package.json:6` scripts `dev`/`build`/`test` (`vitest run`)/`test:watch` (`vitest`). `frontend/vite.config.ts:12` `test.environment=jsdom`, `setupFiles=./src/test/setup.ts`.
- `frontend/vite.config.ts:8` proxy `/api → http://localhost:3000`, port `5173`. `frontend/src/App.tsx:7` fetch `/api/health`.
- Commandes: `devenv shell -- npm --prefix frontend test -- --run`, single `devenv shell -- npm --prefix frontend test -- --run -t "renders title"`. Prérequis `devenv shell -- bash -c 'cd frontend && npm install'` (pas de `node` hors devenv).
- Tests `@testing-library/react` + `jsdom` (`frontend/src/App.test.tsx`). Fichiers `*.test.tsx` à côté du code.

## Lien backend ↔ frontend

- `frontend/vite.config.ts:8` proxy doit matcher `backend/src/main.rs:22` `bind 0.0.0.0:3000`. Changer l'un ⇒ changer l'autre + `justfile:5`.
