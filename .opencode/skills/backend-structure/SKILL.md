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
    mod.rs                 # pub mod <object>; pub use <object>::{Struct, Error}
    <object>/mod.rs        # pub struct Struct + impl Struct pur (serde, no axum/ldap3) — règle 08
    <object>/errors.rs     # pub enum StructError + Display + Error — struct sans error dans mod.rs
  error.rs                 # enum AppError {Ldap,Internal} + Display+Error pur (pas d'axum)
  repository/mod.rs + http/mod.rs + controllers/mod.rs + domain/mod.rs  # pub mod <aggregate>;
```

### Adding a new endpoint

1. `domain/<aggregate>/<object>/mod.rs`: `pub struct Foo` + `domain/<aggregate>/<object>/errors.rs`: `pub enum FooError` (+ Display + Error) + `impl Foo { from_attrs/from_search }` + `#[cfg(test)]` AAA — 1 objet = 1 dossier (règle `08-struct-folder.md`).
2. `domain/<aggregate>/mod.rs`: `pub mod <object>; pub use <object>::{Foo,FooError};` pour ré-export plat (`crate::domain::users::User` reste valide)
3. `repository/ldap.rs`: `pub async fn list_foos() -> Result<Vec<Foo>, AppError> { LdapConnAsync + search + Foo::from_search }` — **seul** fichier avec `ldap3`, pas d'`axum`
4. `controllers/foo.rs`: `pub async fn list() -> Result<Vec<Foo>, AppError> { repository::ldap::list_foos().await }` — thin, pas de `ldap3` direct, pas d'`axum`, pas de `String` nu
5. `http/foo.rs`: `use crate::controllers::foo; pub async fn foo() -> Result<Json<Vec<Foo>>, AppError> { Ok(Json(foo::list().await?)) }` — `axum` seul, `?` propage `AppError`, `IntoResponse` via `http/error.rs`
6. `domain/mod.rs:1` + `repository/mod.rs:1` + `http/mod.rs:1` + `controllers/mod.rs:1` → `pub mod <aggregate>;` + `error.rs` inchangé
7. `lib.rs:7` → `.route("/api/foo", get(http::foo::foo))`
8. Ne pas déplacer `/api/health` sans MAJ `frontend/src/App.tsx` + `frontend/vite.config.ts` proxy

### Domain — 1 objet = 1 dossier (règle 08)

- `domain/<aggregate>/<object>/mod.rs` contient struct + impl + `#[cfg(test)]` — **sans** `pub enum Error` (→ `errors.rs` file sibling). Pas de `#[allow(clippy::module_inception)]`.
- Exemples: `domain/users/uid/` `Uid` + `UidError`; `domain/groups/group/` `Group` + `GroupError`; `domain/groups/members/` `Members` + `MembersError`.
- Objets sans error (ex `domain/health`) → un seul `mod.rs` suffit, pas de `errors.rs`.
- Re-export plat dans `domain/<aggregate>/mod.rs`: `pub use <object>::{Foo,FooError}` pour garder `crate::domain::users::User`.
- VOs avec `String` masqué: `pub struct Uid(String)` — voir règle `05-value-objects.md`.

### Domain impl — méthodes associées, pas de free fns

- Tout comportement lié à un struct va dans `impl Struct`. `http/` ne contient que `axum::Json`, `repository/ldap.rs` seul fait `ldap3`, `controllers/` est thin.
- Nommage: `from_attrs`, `from_search`, `from_ldap`, `new` — `Self`/`Result<Self,FooError>`/`Vec<Self>`, tri/filtrage dedans si déterministe.
- Tests collocalisés `#[cfg(test)]` (AAA) appelle `Foo::from_attrs` directement, pas de mock LDAP.

### Errors — enums, pas de String

- `pub enum FooError {MissingId}` + `Display + Error` propre au domaine, dans `<object>/errors.rs`. `from_attrs` → `Result<Self,FooError>`, `from_search` filtre via `.ok()` et trie.
- `error.rs:5` `pub enum AppError {Ldap(String),Internal(String)}` + `Display + Error` pur (pas d'axum). `http/error.rs:5` y ajoute `IntoResponse (500 Json{error})`. Seul `http/` et `controllers/` l'utilisent.
- Interdit: `Result<_, String>` ou `(StatusCode, Json<Value>)` dans `http/`/`controllers/`, `Option` silencieux sans `FooError` dans `domain/`.

### Axum isolation — seul `main.rs` + `http/`

- Seuls `main.rs` (`axum::serve`) et `http/**` (`Router`, `Json`, `IntoResponse`) importent `axum` en prod. Interdit en prod: `use axum` dans `lib.rs` (hors `#[cfg(test)]`), `controllers/**`, `domain/**`, `error.rs`, `repository/**`. Vérif: `rg -n "use axum|axum\." backend/src --glob '!http/**' --glob '!main.rs' | grep -v "cfg(test)"` doit être vide.

### Ldap isolation — seul `repository/ldap.rs`

- Seul `repository/ldap.rs` importe `ldap3` (LdapConnAsync, Mod, Scope, SearchEntry). Interdit: `use ldap3` dans `controllers/**`, `domain/**`, `http/**`, `error.rs`. `domain/*/from_search` prend `HashMap` pur, pas `SearchEntry`. Vérif: `rg -n "ldap3" backend/src/domain backend/src/controllers backend/src/http backend/src/error.rs` doit être vide.

### Serde contracts

- `serde { derive }` + `serde_json 1` — pas d'autre sérialiseur.
- Domain structs: `#[derive(Serialize, Deserialize)]` + `#[serde(rename_all="camelCase")]` si JSON externe, sinon snake_case par défaut.
- Pas de `serde_json::Value` en handler — typer `Json<T>`.
- Champs optionnels: `Option<T>` + `#[serde(skip_serializing_if="Option::is_none")]`.

### Verification

```bash
devenv shell -- cargo clippy -- -D warnings   # 0 warning
devenv shell -- cargo test                    # collocalisés
rg -n "pub enum.*Error" backend/src --glob '*/mod.rs'   # vide (hors sub-modules)
rg -n "allow\(clippy::module_inception\)" backend/src   # vide
```
