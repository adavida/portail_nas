---
name: frontend-format
description: Enforce frontend formatting (prettier via treefmt) + eslint flat + tsc strict — use when editing frontend/src, vite.config.ts, or any .ts/.svelte.
---

# frontend-format

## Rule — `prettier`/`eslint`/`tsc` via `devenv`, never bare `npx`

```bash
devenv shell -- treefmt --fail-on-change
devenv shell -- bash -c 'cd frontend && npx tsc --noEmit'
devenv shell -- bash -c 'cd frontend && npx svelte-check --tsconfig ./tsconfig.json'
devenv shell -- bash -c 'cd frontend && npx eslint . --max-warnings 0'
```

Never run bare `prettier`/`eslint`/`tsc` outside devenv shell (node 22).

### Enforced

- `prettier` via treefmt — no custom `.prettierrc` (YAGNI); formats `frontend/src/**/*.{ts,svelte}` + `vite.config.ts`.
- `eslint` flat (`typescript-eslint` + `eslint-plugin-svelte` `flat/recommended` + `flat/prettier`, parser TS pour `*.svelte` via `parserOptions.parser`, `eslint.config.js:1`), `--max-warnings 0`. Pas de `eslint-disable` sans `// ponytail:` + justification.
- `tsc` strict — `noEmit`, `skipLibCheck` (`.ts` seuls); `svelte-check` pour les `.svelte` (0 erreur 0 warning).

### Verification

```bash
devenv shell -- treefmt --fail-on-change
devenv shell -- bash -c 'cd frontend && npx tsc --noEmit'
devenv shell -- bash -c 'cd frontend && npx svelte-check --tsconfig ./tsconfig.json'
devenv shell -- bash -c 'cd frontend && npx eslint . --max-warnings 0'
devenv shell -- npm --prefix frontend test -- --run
```
