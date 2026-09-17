# AGENTS.md

> Détails: `.opencode/rules/*.md` chargés via `opencode.json:3` — source of truth.
> Skill: `.opencode/skills/devenv-format/SKILL.md` (trigger `devenv.nix`, `devenv.yaml`).

## Layout

- `devenv.nix` / `devenv.yaml` / `devenv.lock` at repo root
- `backend/src/main.rs`, `frontend/src/` (`App.tsx`, `main.tsx`, `App.test.tsx`, `vite.config.ts`)
- `.vscode/{settings,extensions}.json`, `.gitignore` excludes `.devenv/`, `target/`, `node_modules/`, `frontend/dist/`
