---
name: frontend-structure
description: Enforce frontend file layout main/pages/components + colocalized vitest — use when adding pages, components, or editing frontend/src.
---

# frontend-structure

## Rule — `main.tsx` minimal, `App.tsx` glue, `pages/` data, `components/` pure

```
frontend/src/
  main.tsx                 # boot seul: createRoot → <App />
  App.tsx                  # glue: import Home from './pages/Home' → <Home />
  pages/<Name>.tsx         # page: useState/useEffect/fetch → compose components
  components/<Name>.tsx    # pure: props → JSX, data-testid, no fetch
  pages/<Name>.test.tsx + components/<Name>.test.tsx   # colocalized vitest
  test/setup.ts            # jest-dom once
```

### Adding a page/component

1. `components/Foo.tsx`: `export function Foo({bar}:{bar:string}){return <div data-testid="foo">{bar}</div>}`
2. `pages/FooPage.tsx`: `useState`/`useEffect`/`fetch("/api/foo")` → `<Foo bar={bar} />`
3. `App.tsx` → `import FooPage from './pages/FooPage'` + route/compose
4. Ne pas déplacer `/api/health` sans MAJ `backend/src/lib.rs` + `vite.config.ts` proxy

### Tests collocalisés

`vite.config.ts:12` `environment jsdom` + `globals true` + `@testing-library/react` + `test/setup.ts` — `vi.fn()` pour mock fetch. Pattern:

```tsx
import { render, screen } from '@testing-library/react'
import { HealthBadge } from './HealthBadge'

test('renders health badge', () => {
  render(<HealthBadge status="ok" />)

  expect(screen.getByTestId('health')).toHaveTextContent('backend: ok')
})
```

### Verification

```bash
devenv shell -- npm --prefix frontend test -- --run
devenv shell -- bash -c 'cd frontend && npx tsc --noEmit'
```
