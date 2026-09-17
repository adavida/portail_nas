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
  lib.rs                   # pub mod domain/http/controllers/error/repository + pub use http::router as app (pas de use axum)
  http/mod.rs              # pub mod error/health/users + pub fn router()->Router + pub use Router (axum ici)
  http/<name>.rs           # async fn handler() -> Result<Json<T>, AppError> (axum only, appelle controllers)
  http/error.rs            # impl IntoResponse for AppError (500 Json{error}) — seul fichier avec IntoResponse
  controllers/<name>.rs    # thin -> repository::ldap (pas de ldap3 direct, pas d'axum)
  repository/ldap.rs       # seul endroit avec ldap3 (LdapConnAsync, Mod, Search) + AppError::Ldap, pas d'axum
  domain/<aggregate>/      # répertoire dédié par agrégat (health, users) — 1 objet / fichier
    mod.rs                 # pub mod <object>; pub use <object>::{Struct,Error}
    <object>.rs            # pub struct Struct + pub enum StructError + impl Struct pur (serde, no axum/ldap3)
  error.rs                 # enum AppError {Ldap,Internal} + Display+Error pur (pas d'axum)
  repository/mod.rs + http/mod.rs + controllers/mod.rs + domain/mod.rs  # pub mod <aggregate>;
```

### Adding a new endpoint

1. `domain/<aggregate>/<object>.rs`: `#[derive(Serialize)] pub struct Foo { ... }` + `pub enum FooError {MissingId}` + `impl Foo { pub fn from_attrs(...) -> Result<Self,FooError> / pub fn from_search(...) -> Vec<Self> }` — 1 objet / fichier (ex `domain/users/user.rs:11` `UserError::MissingUid`, `domain/users/new_user.rs:11` `CreateUserError`, `domain/health/mod.rs:3` `Health` — 1 objet → `mod.rs` suffit, `clippy::module_inception` évité)
2. `domain/<aggregate>/mod.rs`: `pub mod <object>; pub use <object>::{Foo,FooError};` pour ré-export plat (`crate::domain::users::User` reste valide)
3. `repository/ldap.rs`: `pub async fn list_foos() -> Result<Vec<Foo>, AppError> { LdapConnAsync + search + Foo::from_search }` — **seul** fichier avec `ldap3`, pas d'`axum`
4. `controllers/foo.rs`: `pub async fn list() -> Result<Vec<Foo>, AppError> { repository::ldap::list_foos().await }` — thin, pas de `ldap3` direct, pas d'`axum`, pas de `String`
5. `http/foo.rs`: `use crate::controllers::foo; pub async fn foo() -> Result<Json<Vec<Foo>>, AppError> { Ok(Json(foo::list().await?)) }` — `axum` seul, `?` propage `AppError::IntoResponse`
6. `domain/mod.rs:1` + `repository/mod.rs:1` + `http/mod.rs:1` + `controllers/mod.rs:1` → `pub mod <aggregate>;` + `error.rs` inchangé
7. `lib.rs:7` → `.route("/api/foo", get(http::foo::foo))`
8. Ne pas déplacer `/api/health` sans MAJ `frontend/src/App.tsx:7` + `frontend/vite.config.ts:8` proxy

### Domain — répertoire dédié + 1 objet / fichier

- `domain/<aggregate>/` répertoire dédié par agrégat (`users`, `health`) — `domain/users/user.rs` + `domain/users/new_user.rs`, `domain/health/mod.rs` (1 objet → `mod.rs`, évite `module_inception`) (ex `domain/users.rs` avec `User+NewUser` interdit).
- 1 fichier = 1 objet : `pub struct Foo` + `pub enum FooError` + `impl Foo` + `#[cfg(test)]` dans le même fichier (ex `domain/users/user.rs:11` `UserError`, `domain/users/new_user.rs:11` `CreateUserError`).
- Re-export `domain/<aggregate>/mod.rs:1` `pub use <object>::{Foo,FooError}` pour garder `crate::domain::users::User` plat.

### Domain impl — méthodes associées, pas de free fns

- Tout comportement du domaine lié à un struct va dans `impl Struct` (`domain/users/user.rs:10` `impl User`). `http/` ne contient que `axum::Json`, `repository/ldap.rs` seul fait `ldap3`, `controllers/` est thin.
- Nommage: `from_attrs`, `from_search`, `from_ldap`, `new` — `Self`/`Result<Self,FooError>`/`Vec<Self>`, tri/filtrage dedans si déterministe.
- Test collocalisé `domain/<aggregate>/<object>.rs:60` `#[cfg(test)]` appelle `Foo::from_attrs` directement, pas de mock LDAP.

### Errors — enums, pas de String

- `domain/<aggregate>/<object>.rs:11` `pub enum FooError {MissingId}` + `Display + Error` propre au domaine (`domain/users/user.rs:11` `UserError::MissingUid`). `from_attrs` retourne `Result<Self,FooError>`, `from_search` filtre via `.ok()` et trie.
- `error.rs:5` `pub enum AppError {Ldap(String),Internal(String)}` + `Display + Error` pur (pas d'axum) — `http/error.rs:5` y ajoute `IntoResponse (500 Json{error})`. Seul `http/` et `controllers/` l'utilisent (`http/users.rs:5` `Result<Json<Vec<User>>, AppError>`, `controllers/users.rs:29` `Result<Vec<User>,AppError>` avec `map_err(|e| AppError::Ldap(e.to_string()))`).
- Interdit: `Result<_, String>` ou `(StatusCode, Json<Value>)` dans `http/`/`controllers/`, `Option` silencieux sans `FooError` dans `domain/`.

### Axum isolation — seul `main.rs` + `http/`

- Seuls `backend/src/main.rs:8` `axum::serve` et `backend/src/http/**` (`http/mod.rs:5` `Router`, `http/health.rs:1` `Json`, `http/users.rs:1` `Json`, `http/error.rs:1` `IntoResponse`) importent `axum` en prod.
- Interdit en prod: `use axum` dans `lib.rs` (hors `#[cfg(test)]`), `controllers/**`, `domain/**`, `error.rs`, `repository/**`. `lib.rs:11` `#[cfg(test)]` peut utiliser `axum::body/http` + `tower` pour `oneshot`. Vérif: `rg -n "use axum|axum::" backend/src --glob '!http/**' --glob '!main.rs' | grep -v "cfg(test)"` doit être vide.

### Ldap isolation — seul `repository/ldap.rs`

- Seul `backend/src/repository/ldap.rs:1` importe `ldap3` (`LdapConnAsync, Mod, Scope, SearchEntry`). Interdit: `use ldap3` dans `controllers/**`, `domain/**`, `http/**`, `error.rs` ( `domain/users/user.rs:44` `from_search` prend `HashMap` pur, pas `SearchEntry`). `lib.rs:84` `#[cfg(test)]` seed peut utiliser `ldap3` pour `add`. Vérif: `rg -n "ldap3" backend/src/domain backend/src/controllers backend/src/http backend/src/error.rs` doit être vide.
- `lib.rs:6` `pub use http::router as app` — pas de `use axum` direct en prod.

### Serde contracts

- `backend/Cargo.toml:9` `serde { derive }` + `serde_json 1` — pas d'autre sérialiseur.
- Domain structs: `#[derive(Serialize, Deserialize)]` + `#[serde(rename_all="camelCase")]` si JSON externe, sinon `snake_case` par défaut.
- Pas de `serde_json::Value` en retour handler — typer `Json<T>` (ex `http/health.rs:5` `Json<Health>`).
- Champs optionnels: `Option<T>` + `#[serde(skip_serializing_if="Option::is_none")]`.

### Tests — assertions via helpers `assert_auth_*`

- `repository/ldap/users.rs:155` `assert_auth_ok(uid,password)` / `assert_auth_fail` / `assert_auth_err` — **seules** assertions autorisées pour `authenticate_user`. Interdit: `assert!(authenticate_user(...).await.unwrap())` ou `assert!(!...)` direct.
- Helpers 2 params `uid:&str, password:&str` → `authenticate_user(uid.to_string(), password.to_string()).await.unwrap()` + `assert!( )` / `assert!(! )` / `assert!(is_err())`.
- Tests `repository/ldap/users.rs:175` utilisent `assert_auth_ok(&uid,"secret123")` `assert_auth_fail(&uid,"wrong")` `assert_auth_err(&uid,"")` — 0 `authenticate_user` direct hors helpers. Vérif: `rg -n "authenticate_user" backend/src/repository/ldap/users.rs | grep -v "fn assert_auth" | grep -v "pub async fn authenticate" | grep "assert!"` doit être vide.

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
