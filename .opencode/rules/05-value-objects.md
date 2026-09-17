# 05 — Value Objects

> Skills de référence:
>
> - Structure: `.opencode/skills/backend-structure/SKILL.md` (trigger `backend/src/domain/`)
> - Formatage: `.opencode/skills/backend-format/SKILL.md` (trigger `*.rs`)

## Objectif

Remplacer les `String` bruts pour tout concept métier à invariants (identifiant, nom, email, secret, etc.) par des Value Objects qui centralisent validation, invariants et sérialisation. Plus de `String` nu dans `domain/<aggregate>/*` pour un concept couvert par un VO.

YAGNI: pas de VO pour un `String` sans invariant.

## Localisation — 1 VO = 1 fichier

```
backend/src/domain/<aggregate>/
  <vo>.rs     # pub struct Foo(String) + FooError
  mod.rs      # pub mod <vo>; pub use <vo>::Foo;
  <entity>.rs # utilise Foo (pas String)
```

Exemple actuel `users`:
`uid.rs`/`name.rs`/`email.rs`/`password.rs` + `user.rs`/`new_user.rs`.

Respecte [backend-structure](/.opencode/skills/backend-structure/SKILL.md): `domain/<aggregate>/` + 1 fichier = 1 objet (`pub struct` + `pub enum Error` + `impl` + `#[cfg(test)]`).

Interdit: `backend/src/domain/value_objects/` ou `domain/shared/` global — VOs restent dans leur agrégat; extraire seulement quand 2e agrégat réutilise le même concept.

## Invariants

Chaque VO définit ses invariants dans `try_new` — seul endroit autorisé à tightener. Aucun `validate()` dispersé en entity/controllers/http.

Exemple (illustratif, non normatif):

| VO         | Invariant exemple                                                | Erreur                              |
| ---------- | ---------------------------------------------------------------- | ----------------------------------- |
| `Uid`      | `trim().is_empty()==false` + `len 2..32` + `^[a-z0-9._-]+$`      | `UidError::Empty` / `InvalidFormat` |
| `Name`     | `trim().is_empty()==false` + `len 1..100`                        | `NameError::Empty`                  |
| `Email`    | `""` vide autorisé sinon `contains('@') && contains('.')` + ≤254 | `EmailError::InvalidFormat`         |
| `Password` | `is_empty()==false` (future: `len>=8`)                           | `PasswordError::Empty`              |

Règle: tout tightening reste dans `VO::try_new`.

## Pattern — tuple struct privée + `try_new` + `Deserialize` validante

```rust
// backend/src/domain/<aggregate>/<vo>.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Foo(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FooError { Empty, InvalidFormat }

impl std::fmt::Display for FooError { /* ... */ }
impl std::error::Error for FooError {}

impl Foo {
    pub fn try_new(raw: String) -> Result<Self, FooError> {
        let s = raw.trim().to_string();
        if s.is_empty() { return Err(FooError::Empty); }
        // invariants spécifiques ici
        Ok(Self(s))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}
impl<'de> Deserialize<'de> for Foo {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}
```

Variantes:

- VO qui autorise vide (ex: `Email`) → `try_new("") == Ok(Self(""))`.
- VO secret (ex: `Password`) → ne dérive pas `Serialize` (jamais en réponse).

## Intégration domaine

- Entity/DTO: `Struct { field: Foo }` au lieu de `field: String` — `Serialize` transparent donc JSON reste `string`.
- Construction `Foo::try_new` dans `from_attrs`/`from_ldap` et autres factories.
- Conversion vers infra via `foo.as_str()` (`dn()`, `to_attrs()`, requêtes LDAP).
- Plus de `validate()` manuel: `Deserialize` appelle `try_new`.

## Contrats serde

- `Serialize` = `#[serde(transparent)]` → JSON reste `string`.
- `Deserialize` appelle `try_new` — pas de `String` intermédiaire qui bypass.

## Interdits

- `String` pour un concept couvert par VO dans `domain/<aggregate>/*`, `controllers/*`, `http/*`, `repository/*` — utiliser le VO. Vérif générique: `rg -n ": String" backend/src/domain/<aggregate> backend/src/controllers backend/src/http backend/src/repository` doit être vide hors `pub struct Foo(String)` interne au VO et `main.rs`/`#[cfg(test)]`.
- `validate()` libre hors VO — toute règle métier va dans `VO::try_new`.
- `unwrap()`/`expect()` hors `main.rs`/`#[cfg(test)]`.

## Tests — collocalisés AAA

Chaque `<vo>.rs` a `#[cfg(test)]` AAA (valid/invalid/trim/len).

## Vérification

```bash
devenv shell -- cargo test -- --nocapture
devenv shell -- cargo clippy -- -D warnings
devenv shell -- cargo fmt -- --check
devenv shell -- treefmt --fail-on-change
rg -n ": String" backend/src/domain backend/src/controllers backend/src/http backend/src/repository
```
