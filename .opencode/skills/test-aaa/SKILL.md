---
name: test-aaa
description: Enforce Arrange-Act-Assert with blank line between sections for all tests — use when writing or editing tests in backend (Rust) or frontend (vitest).
---

# test-aaa

## Rule — AAA avec ligne vide entre les 3 parties

Rust:

```rust
#[test]
fn example() {
    let input = "a";

    let result = do_something(input);

    assert_eq!(result, "b");
}
```

Frontend:

```tsx
test("example", () => {
  render(<Comp />);

  fireEvent.click(screen.getByTestId("btn"));

  expect(screen.getByTestId("out")).toBeInTheDocument();
});
```

- **Arrange** : `let`, `render`, `globalThis.fetch = vi.fn`, `gen_uid`, `ensure_clean`
- **Act** : l'appel testé (`User::from_attrs`, `create_user`, `app.oneshot`, `fireEvent`, `fetch`)
- **Assert** : `assert_eq!`, `expect`, `assert_auth_*`

Vérif : chaque `#[test]` / `test(` contient `\n\n` entre sections. Pas de `let x = foo(); assert_eq!(x, ...)` consécutifs sans ligne vide.

## Assertions faciles à lire

- **Message explicite** : `assert_eq!(actual, expected, "contexte")` / `assert!(cond, "message")`; en TS variable intermédiaire `expect(actual).toHaveTextContent(...)` — jamais `assert!(a == b)` nue.
- **Variable intermédiaire** : `let result = foo().unwrap_err(); assert_eq!(result, E::MissingUid, "...")` — pas d'appel inline.
- **1 assertion = 1 intention** : pas de `assert!(a && b)` composite.
- **Helpers nommés** : `assert_auth_ok(&uid, "pwd")` / `assert_auth_fail` / `assert_auth_err` pour `authenticate_user` — jamais d'appel direct répété.
