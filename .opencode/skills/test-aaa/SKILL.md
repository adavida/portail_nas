---
name: test-aaa
description: Enforce Arrange-Act-Assert with blank line between sections for all tests — use when writing or editing tests in backend (Rust) or frontend (vitest).
---

# test-aaa

## Rule — AAA avec ligne vide entre les 3 parties

Tout test suit **Arrange, Act, Assert** séparés par **une ligne vide** :

```rust
#[test]
fn example() {
    let input = "a"; // Arrange

    let result = do_something(input); // Act

    assert_eq!(result, "b"); // Assert
}
```

```tsx
test("example", () => {
  render(<Comp />); // Arrange

  fireEvent.click(screen.getByTestId("btn")); // Act

  expect(screen.getByTestId("out")).toBeInTheDocument(); // Assert
});
```

- **Arrange** : `let`, `render`, `globalThis.fetch = vi.fn`, `gen_uid`, `ensure_clean`
- **Act** : l'appel testé (`User::from_attrs`, `create_user`, `app.oneshot`, `fireEvent`, `fetch`)
- **Assert** : `assert_eq!`, `expect`, `assert_auth_*`

Vérif : chaque `#[test]` / `test(` doit contenir `\n\n` entre Arrange/Act et Act/Assert. Pas de `let x = foo(); assert_eq!(x, ...)` sur lignes consécutives sans ligne vide.

Source of truth : `backend/src/domain/users/user.rs:64` `backend/src/repository/ldap/users.rs:180` `frontend/src/components/UsersTable.test.tsx:5`
