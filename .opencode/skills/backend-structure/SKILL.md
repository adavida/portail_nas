---
name: backend-structure
description: Enforce backend file layout domain/routes + serde JSON contracts — use when adding routes, domain logic, or serde types in backend/src.
---

# backend-structure

## Rule — `main.rs` minimal, `lib.rs` router, `routes/` handlers, `domain/` pure

Source of truth: `backend/src/lib.rs:6` + `backend/src/main.rs:4`.

```
backend/src/
  main.rs        # boot seul: app() + TcpListener 0.0.0.0:3000
  lib.rs         # pub mod domain/routes + app() -> Router
  routes/<name>.rs  # async fn handler() -> Json<T> (axum only)
  domain/<name>.rs  # struct + fn pure (serde, no axum)
  routes/mod.rs + domain/mod.rs  # pub mod <name>;
```

### Adding a new endpoint

1. `domain/foo.rs`: `#[derive(Serialize, Deserialize)] pub struct Foo { ... }` + `pub fn foo_logic() -> Foo`
2. `routes/foo.rs`: `use crate::domain::foo::{Foo, foo_logic}; pub async fn foo() -> Json<Foo> { Json(foo_logic()) }`
3. `domain/mod.rs:1` + `routes/mod.rs:1` → `pub mod foo;`
4. `lib.rs:7` → `.route("/api/foo", get(routes::foo::foo))`
5. Ne pas déplacer `/api/health` sans MAJ `frontend/src/App.tsx:7` + `frontend/vite.config.ts:8` proxy

### Serde contracts

- `backend/Cargo.toml:9` `serde { derive }` + `serde_json 1` — pas d'autre sérialiseur.
- Domain structs: `#[derive(Serialize, Deserialize)]` + `#[serde(rename_all="camelCase")]` si JSON externe, sinon `snake_case` par défaut.
- Pas de `serde_json::Value` en retour handler — typer `Json<T>` (ex `routes/health.rs:5` `Json<Health>`).
- Champs optionnels: `Option<T>` + `#[serde(skip_serializing_if="Option::is_none")]`.

### Tests collocalisés

`lib.rs:11` `#[cfg(test)]` `#[tokio::test]` + `tower::ServiceExt::oneshot` + `http_body_util::BodyExt` — pas de vrai LDAP (mock `LDAP_TEST_*`).

```bash
devenv shell -- cargo test health_returns_ok -- --nocapture
```

### Verification

```bash
devenv shell -- cargo test
devenv shell -- treefmt
```
