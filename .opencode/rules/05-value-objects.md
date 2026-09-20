# 05 — Value Objects

> Skills: `backend-structure` (trigger `backend/src/domain/`), `backend-format` (trigger `*.rs`), `08-struct-folder.md` pour le layout fichiers.

## Objectif

Tout `String` métier à invariants (identifiant, nom, email, secret…) devient un VO dans son agrégat — plus de `String` nu pour un concept couvert. YAGNI: pas de VO pour un `String` sans invariant.

## Localisation — 1 VO = 1 dossier (règle 08)

Layout: `domain/<aggregate>/<vo>/{mod.rs,error.rs}` (cf. 08); ex `users`: VO `uid/`, `name/`, `email/`, `password/` + `user.rs`/`new_user.rs`.

Interdit: `domain/value_objects/` ou `domain/shared/` global — VOs restent dans leur agrégat; extraire seulement quand 2e agrégat réutilise le même concept.

## Invariants — tout dans `try_new`

Seul `VO::try_new` valide/tighten. Aucun `validate()` dispersé. (Exemples actuels, illustratifs non normatifs: `Uid` trim+`len 2..32`+`^[a-z0-9._-]+$`; `Name` trim+`len 1..100`; `Email` vide autorisé sinon `@`+`.`+≤254; `Password` non vide.)

## Pattern

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Foo(String);

impl Foo {
    pub fn try_new(raw: String) -> Result<Self, FooError> {
        let s = raw.trim().to_string();
        if s.is_empty() { return Err(FooError::Empty); }
        Ok(Self(s))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl<'de> Deserialize<'de> for Foo {
    fn deserialize<D>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}
```

Variantes: `Email` → `try_new("") == Ok`. VO secret (`Password`) → pas de `Serialize` (jamais en réponse).

## Intégration & serde

- Entity/DTO: `field: Foo` au lieu de `String` — `Serialize` transparent, JSON reste `string`.
- Construction `Foo::try_new` dans `from_attrs`/`from_ldap`/factories; conversion infra via `foo.as_str()`.
- `Deserialize` appelle `try_new` — pas de `String` intermédiaire qui bypass; plus de `validate()` manuel.

## Interdits

- `String` pour un concept couvert par VO (vérif): `rg -n ": String" backend/src/{domain,controllers,http,repository}` vide hors `pub struct Foo(String)` interne au VO et `#[cfg(test)]`.
- `validate()` libre hors VO; `unwrap()`/`expect()` hors `main.rs`/`#[cfg(test)]`.

## Tests & vérif

Chaque `<vo>/mod.rs` a `#[cfg(test)]` AAA (valid/invalid/trim/len).

```bash
devenv shell -- cargo test && devenv shell -- cargo clippy -- -D warnings
rg -n ": String" backend/src/domain backend/src/controllers backend/src/http backend/src/repository
```
