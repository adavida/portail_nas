---
name: frontend-format
description: Enforce frontend formatting (prettier via treefmt) + eslint flat + tsc strict — use when editing frontend/src, vite.config.ts, or any .ts/.tsx.
---

# frontend-format

## Rule — `prettier`/`eslint`/`tsc` via `devenv`, never bare `npx`

Executable source of truth: `devenv.nix:127` + `frontend/package.json:6` + `frontend/eslint.config.js`.

### How to format/lint

```bash
devenv shell -- treefmt              # prettier via treefmt-nix, also on enterShell
devenv shell -- bash -c 'cd frontend && npx tsc --noEmit'  # strict: noEmit, skipLibCheck
devenv shell -- bash -c 'cd frontend && npx eslint . --max-warnings 0'
devenv shell -- bash -c 'cd frontend && npx eslint . --fix' # auto-fix
```

Never run `prettier`/`eslint`/`tsc` outside `devenv shell` (node 22).

### What `prettier` enforces (treefmt)

- `treefmt.config.programs.prettier.enable` — no custom `.prettierrc` (YAGNI), defaults `semi, singleQuote`.
- `projectRootFile = "devenv.nix"` — formats `frontend/src/**/*.{ts,tsx}` + `vite.config.ts`.

### What `eslint` enforces (flat, CI fails if warnings)

1. `typescript-eslint` recommended + `react-hooks` + `react-refresh` (`frontend/eslint.config.js:1`).
2. `parserOptions.projectService: true` aligns `tsconfig.json:14` `strict true`.
3. `--max-warnings 0` — 0 warnings allowed, pas de `eslint-disable` sans `// ponytail:` + justification.

### Verification

```bash
devenv shell -- treefmt --fail-on-change
devenv shell -- bash -c 'cd frontend && npx tsc --noEmit'
devenv shell -- bash -c 'cd frontend && npx eslint . --max-warnings 0'
devenv shell -- npm --prefix frontend test -- --run
```
