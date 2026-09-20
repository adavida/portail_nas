---
name: frontend-format
description: Enforce frontend formatting (prettier via treefmt) + eslint flat + tsc strict — use when editing frontend/src, vite.config.ts, or any .ts/.tsx.
---

# frontend-format

## Rule — `prettier`/`eslint`/`tsc` via `devenv`, never bare `npx`

```bash
devenv shell -- treefmt --fail-on-change
devenv shell -- bash -c 'cd frontend && npx tsc --noEmit'
devenv shell -- bash -c 'cd frontend && npx eslint . --max-warnings 0'
```

Never run bare `prettier`/`eslint`/`tsc` outside devenv shell (node 22).

### Enforced

- `prettier` via treefmt — no custom `.prettierrc` (YAGNI); formats `frontend/src/**/*.{ts,tsx}` + `vite.config.ts`.
- `eslint` flat (`typescript-eslint` + `react-hooks` + `react-refresh`, `eslin.config.js:1`), `projectService: true` aligné `tsconfig.json:14` `strict: true`, `--max-warnings 0`. Pas de `eslint-disable` sans `// ponytail:` + justification.
- `tsc` strict — `noEmit`, `skipLibCheck`.

### Verification

```bash
devenv shell -- treefmt --fail-on-change
devenv shell -- bash -c 'cd frontend && npx tsc --noEmit'
devenv shell -- bash -c 'cd frontend && npx eslint . --max-warnings 0'
devenv shell -- npm --prefix frontend test -- --run
```
