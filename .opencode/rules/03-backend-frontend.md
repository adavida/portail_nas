# 03 — Backend & Frontend

> Skills de référence:
>
> - Formatage/lints: `.opencode/skills/backend-format/SKILL.md` (trigger `backend/src`, `Cargo.toml`, `*.rs`)
> - Structure/serde: `.opencode/skills/backend-structure/SKILL.md` (trigger `backend/src/lib.rs`, `routes/`, `domain/`)

## Backend `portail-backend` (Axum 0.7, Tokio full)

- Workspace `Cargo.toml:2` `members = ["backend"]`, crate `backend/Cargo.toml:2`. Voir skills ci-dessus pour `rustfmt`/`clippy`/`serde`/`domain`/`routes`.

## Frontend `portail-frontend` (Vite + React 18 + TS)

> Skills de référence:
>
> - Formatage/lints: `.opencode/skills/frontend-format/SKILL.md` (trigger `frontend/src`, `*.tsx`, `eslint.config.js`)
> - Structure: `.opencode/skills/frontend-structure/SKILL.md` (trigger `frontend/src/main.tsx`, `pages/`, `components/`)

- `frontend/package.json:6` scripts `dev`/`build`/`test` (`vitest run`)/`test:watch` (`vitest`). `frontend/vite.config.ts:12` `test.environment=jsdom`, `setupFiles=./src/test/setup.ts`.
- `frontend/vite.config.ts:8` proxy `/api → http://localhost:3000`, port `5173`. `frontend/src/pages/Home.tsx:7` fetch `/api/health` → `components/HealthBadge.tsx:1` pure. `App.tsx:1` glue `pages/Home`.
- Commandes: `devenv shell -- npm --prefix frontend test -- --run`, single `devenv shell -- npm --prefix frontend test -- --run -t "renders title"`. Prérequis `devenv shell -- bash -c 'cd frontend && npm install'` (pas de `node` hors devenv).
- Tests collocalisés `@testing-library/react` + `jsdom` (`pages/Home.test.tsx`, `components/HealthBadge.test.tsx`, `App.test.tsx` legacy).

## Lien backend ↔ frontend

- `frontend/vite.config.ts:8` proxy doit matcher `backend/src/main.rs:22` `bind 0.0.0.0:3000`. Changer l'un ⇒ changer l'autre + `justfile:5`.
