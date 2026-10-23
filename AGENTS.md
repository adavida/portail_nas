# AGENTS.md

> Détails: `.opencode/rules/*.md` chargés via `opencode.json:3` — source of truth.
> Skill: `.opencode/skills/devenv-format/SKILL.md` (trigger `devenv.nix`, `devenv.yaml`).

## Layout

- `devenv.nix` / `devenv.yaml` / `devenv.lock` at repo root
- `backend/{core,directory,oidc,server}/src/` (crates: core=env+AppError, directory=users/groups+LDAP, oidc=provider+auth, server=assemblage+bin), `frontend/src/` (`App.svelte`, `main.ts`, `App.test.ts`, `vite.config.ts`)
- `.vscode/{settings,extensions}.json`, `.gitignore` excludes `.devenv/`, `target/`, `node_modules/`, `frontend/dist/`
