---
name: frontend-structure
description: Enforce frontend file layout main/pages/components + colocalized vitest — use when adding pages, components, or editing frontend/src.
---

# frontend-structure

## Rule — `main.tsx` minimal, `App.tsx` glue, `pages/` data, `components/` pure

Source of truth: `frontend/src/main.tsx:1` + `frontend/src/App.tsx:1` + `frontend/vite.config.ts:8`.

```
frontend/src/
  main.tsx                 # boot seul: createRoot → <App />
  App.tsx                  # glue: import Home from './pages/Home' → <Home />
  pages/<Name>.tsx         # page: useState/useEffect/fetch → compose components
  components/<Name>.tsx    # pure: props → JSX, data-testid, no fetch
  pages/<Name>.test.tsx + components/<Name>.test.tsx  # colocalized vitest
  test/setup.ts            # jest-dom once
```

### Adding a new page/component

1. `components/Foo.tsx`: `export function Foo({bar}:{bar:string}){return <div data-testid="foo">{bar}</div>}`
2. `pages/FooPage.tsx`: `import {useEffect,useState} from 'react'; import {Foo} from '../components/Foo'; export default function FooPage(){const [bar,setBar]=useState(""); useEffect(()=>{fetch("/api/foo").then(r=>r.json()).then(d=>setBar(d.bar))},[]); return <Foo bar={bar} />}`
3. `App.tsx` → `import FooPage from './pages/FooPage'` + route/compose
4. Ne pas déplacer `/api/health` sans MAJ `backend/src/lib.rs:7` + `vite.config.ts:8` proxy

### Tests collocalisés

`vite.config.ts:12` `environment jsdom` + `globals true` + `@testing-library/react` — `vi.fn()` pour mock `fetch`.

```tsx
// components/HealthBadge.test.tsx
import {render,screen} from '@testing-library/react'; import {HealthBadge} from './HealthBadge'; test('renders',{render(<HealthBadge status="ok"/>); expect(screen.getByTestId('health')).toHaveTextContent('backend: ok')})
```

### Verification

```bash
devenv shell -- npm --prefix frontend test -- --run
devenv shell -- bash -c 'cd frontend && npx tsc --noEmit'
```
