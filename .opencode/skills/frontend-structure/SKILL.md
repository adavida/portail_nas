---
name: frontend-structure
description: Enforce frontend file layout main/pages/components + colocalized vitest — use when adding pages, components, or editing frontend/src.
---

# frontend-structure

## Rule — `main.ts` minimal, `App.svelte` glue, `pages/` data, `components/` pure, `api/` fetch seul

```
frontend/src/
  main.ts                  # boot seul: mount(App) → App.svelte
  App.svelte               # glue: import Home from './pages/Home.svelte' → <Home />
  api/<ressource>.ts       # seul endroit avec fetch("/api/...") (users, groups, auth) + request<T> via api/http.ts
  pages/<Name>.svelte      # page: $state/$derived + onMount → compose components
  components/<Name>.svelte # pure: $props → markup, data-testid, no fetch
  pages/<Name>.test.ts + components/<Name>.test.ts   # colocalized vitest
  test/setup.ts            # jest-dom + reset store toasts
```

### Adding a page/component

1. `api/foo.ts`: `export function listFoos() { return request<Foo[]>("GET", "/api/foos"); }` (1 ressource = 1 fichier, fetch uniquement ici)
2. `components/Foo.svelte`: `let { bar }: { bar: string } = $props();` + `<div data-testid="foo">{bar}</div>`
3. `pages/FooPage.svelte`: `onMount` + `listFoos()` de `../api/foo` → `<Foo {bar} />`
4. `App.svelte` → `import FooPage from './pages/FooPage.svelte'` + compose
5. Ne pas déplacer `/api/health` sans MAJ `backend/src/lib.rs` + `vite.config.ts` proxy

### Tests collocalisés

`vite.config.ts` `environment jsdom` + `globals true` + `svelteTesting()` (plugin sous `VITEST`) + `@testing-library/svelte` + `test/setup.ts` — `vi.fn()` pour mock fetch. `await fireEvent.*` (le DOM svelte flush en microtask), inputs via `fireEvent.input` (bind:value). Pattern:

```ts
import { fireEvent, render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import HealthBadge from "./HealthBadge.svelte";

test("renders health badge", async () => {
  render(HealthBadge, { props: { status: "ok" } });

  await fireEvent.click(screen.getByTestId("btn"));

  expect(screen.getByTestId("health")).toHaveTextContent("backend: ok");
});
```

### Verification

```bash
devenv shell -- npm --prefix frontend test -- --run
devenv shell -- bash -c 'cd frontend && npx svelte-check --tsconfig ./tsconfig.json'
```
