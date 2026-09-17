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

## Rule — Assertions faciles à lire

- **Message explicite** : toujours `assert_eq!(actual, expected, "contexte lisible")` / `assert!(cond, "message")` en Rust, `const actual = ...; expect(actual).toHaveTextContent(...)` en TS — jamais `assert!(a == b)` sans message
- **Variable intermédiaire** : `let result = foo().unwrap_err(); assert_eq!(result, Err::MissingUid, "empty uid should be rejected")` pas `assert_eq!(foo().unwrap_err(), ...)` inline
- **1 assertion = 1 intention** : `assert_eq!(u.name, "Alice D", "displayName should win over cn")` pas `assert!(u.name == "Alice D" && u.email == "")`
- **Helpers nommés** : `assert_auth_ok(&uid, "pwd")` / `assert_auth_fail` / `assert_auth_err` au lieu de `assert!(authenticate_user(...).await.unwrap())` répété

Source of truth : `backend/src/domain/users/user.rs:64` `backend/src/repository/ldap/users.rs:180` `frontend/src/components/UsersTable.test.tsx:5` `backend/src/domain/users/new_user.rs:83`
