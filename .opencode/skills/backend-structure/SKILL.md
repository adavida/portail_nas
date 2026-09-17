---
name: backend-structure
description: Enforce backend file layout domain/http/controllers + serde JSON contracts — use when adding http handlers, controllers, or domain types in backend/src.
---

# backend-structure

## Rule — `main.rs` + `http/` seuls axum, `lib.rs` re-export, `controllers/`/`domain`/`error.rs` sans axum

Source of truth: `backend/src/main.rs:4` + `backend/src/http/mod.rs:1` + `backend/src/lib.rs:6` re-export.

```
backend/src/
  main.rs                  # boot seul: app() + TcpListener + axum::serve (seul avec http/ a le droit d'importer axum)
  lib.rs                   # pub mod domain/http/controllers/error + pub use http::router as app (pas de use axum)
  http/mod.rs              # pub mod error/health/users + pub fn router()->Router + pub use Router (axum ici)
  http/<name>.rs           # async fn handler() -> Result<Json<T>, AppError> (axum only, appelle controllers)
  http/error.rs            # impl IntoResponse for AppError (500 Json{error}) — seul fichier avec IntoResponse
  controllers/<name>.rs    # orchestration I/O ldap3, map erreurs -> AppError::Ldap (pas d'axum)
  domain/<name>.rs         # struct + impl + enum Error pur (serde, no axum/ldap3)
  error.rs                 # enum AppError {Ldap,Internal} + Display+Error pur (pas d'axum)
  http/mod.rs + controllers/mod.rs + domain/mod.rs  # pub mod <name>;
```

### Adding a new endpoint

1. `domain/foo.rs`: `#[derive(Serialize)] pub struct Foo { ... }` + `pub enum FooError {MissingId}` + `impl Foo { pub fn from_attrs(...) -> Result<Self,FooError> / pub fn from_search(...) -> Vec<Self> }` — pas de `pub fn foo_logic` libre (ex `domain/users.rs:11` `UserError::MissingUid`, `User::from_attrs:Result`)
2. `controllers/foo.rs`: `pub async fn list() -> Result<Vec<Foo>, AppError> { ldap search.map_err(|e| AppError::Ldap(e.to_string()))? + Foo::from_search }` — I/O `ldap3` seul, pas d'`axum`, pas de `String`
3. `http/foo.rs`: `use crate::controllers::foo; pub async fn foo() -> Result<Json<Vec<Foo>>, AppError> { Ok(Json(foo::list().await?)) }` — `axum` seul, `?` propage `AppError::IntoResponse`
4. `domain/mod.rs:1` + `http/mod.rs:1` + `controllers/mod.rs:1` → `pub mod foo;` + `error.rs` inchangé
5. `lib.rs:7` → `.route("/api/foo", get(http::foo::foo))`
6. Ne pas déplacer `/api/health` sans MAJ `frontend/src/App.tsx:7` + `frontend/vite.config.ts:8` proxy

### Domain impl — méthodes associées, pas de free fns

- Tout comportement du domaine lié à un struct va dans `impl Struct` (`domain/users.rs:10` `impl User`). `http/` ne contient que `axum::Json`, `controllers/` seul fait `ldap3`.
- Nommage: `from_attrs`, `from_search`, `from_ldap`, `new` — `Self`/`Result<Self,FooError>`/`Vec<Self>`, tri/filtrage dedans si déterministe.
- Test collocalisé `domain/foo.rs:43` `#[cfg(test)]` appelle `Foo::from_attrs` directement, pas de mock LDAP.

### Errors — enums, pas de String

- `domain/<name>.rs:11` `pub enum FooError {MissingId}` + `Display + Error` propre au domaine (`domain/users.rs:11` `UserError::MissingUid`). `from_attrs` retourne `Result<Self,FooError>`, `from_search` filtre via `.ok()` et trie.
- `error.rs:5` `pub enum AppError {Ldap(String),Internal(String)}` + `Display + Error` pur (pas d'axum) — `http/error.rs:5` y ajoute `IntoResponse (500 Json{error})`. Seul `http/` et `controllers/` l'utilisent (`http/users.rs:5` `Result<Json<Vec<User>>, AppError>`, `controllers/users.rs:29` `Result<Vec<User>,AppError>` avec `map_err(|e| AppError::Ldap(e.to_string()))`).
- Interdit: `Result<_, String>` ou `(StatusCode, Json<Value>)` dans `http/`/`controllers/`, `Option` silencieux sans `FooError` dans `domain/`.

### Axum isolation — seul `main.rs` + `http/`

- Seuls `backend/src/main.rs:8` `axum::serve` et `backend/src/http/**` (`http/mod.rs:5` `Router`, `http/health.rs:1` `Json`, `http/users.rs:1` `Json`, `http/error.rs:1` `IntoResponse`) importent `axum` en prod.
- Interdit en prod: `use axum` dans `lib.rs` (hors `#[cfg(test)]`), `controllers/**`, `domain/**`, `error.rs`. `lib.rs:11` `#[cfg(test)]` peut utiliser `axum::body/http` + `tower` pour `oneshot`. Vérif: `rg -n "use axum|axum::" backend/src --glob '!http/**' --glob '!main.rs' | grep -v "cfg(test)"` doit être vide.
- `lib.rs:6` `pub use http::router as app` — pas de `use axum` direct en prod.

### Serde contracts

- `backend/Cargo.toml:9` `serde { derive }` + `serde_json 1` — pas d'autre sérialiseur.
- Domain structs: `#[derive(Serialize, Deserialize)]` + `#[serde(rename_all="camelCase")]` si JSON externe, sinon `snake_case` par défaut.
- Pas de `serde_json::Value` en retour handler — typer `Json<T>` (ex `http/health.rs:5` `Json<Health>`).
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
