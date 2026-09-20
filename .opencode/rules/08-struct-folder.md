# 08 — Struct Folder

> Skills de référence:
>
> - Structure: `.opencode/skills/backend-structure/SKILL.md` (trigger `backend/src/`)

## Objectif

1 objet = 1 fichier pour la struct, 1 fichier `errors.rs` séparé (snake_case) pour ses erreurs dans son sous-dossier — zéro warning `clippy::module_inception`.

## Règle

- `backend/src/<module>/<object>/mod.rs` contient `pub struct <Struct>` + `impl <Struct> { try_new/create/from_* }` + `#[cfg(test)]` — **pas** de `Error` dedans (évite `backend/src/<module>/<object>/<object>.rs` → `module_inception`).
- `backend/src/<module>/<object>/errors.rs` contient `pub enum <Struct>Error` + `Display` + `Error` (snake_case, minuscules).
- `backend/src/<module>/<object>/mod.rs` ré-exporte `pub mod errors; pub use errors::<Struct>Error;` et le `mod.rs` parent fait `pub mod <object>; pub use <object>::{<Struct>,<Struct>Error};` pour garder `crate::domain::groups::Group` plat.
- Si `backend/src/<module>.rs` existe (ex: `env.rs` à la racine), le split devient `backend/src/<module>/mod.rs` (struct) + `backend/src/<module>/errors.rs`.
- Frontend identique snake_case: `frontend/src/<module>/<object>/errors.ts`.

Exemples:

- `backend/src/env.rs` → `backend/src/env/mod.rs` (`Env`) + `backend/src/env/errors.rs` (`EnvError`)
- `backend/src/domain/groups/group.rs` → `backend/src/domain/groups/group/mod.rs` (`Group`) + `backend/src/domain/groups/group/errors.rs` (`GroupError`)
- `backend/src/domain/groups/members.rs` → `backend/src/domain/groups/members/mod.rs` (`Members`) + `errors.rs`
- VO suit même split: `backend/src/domain/users/uid.rs` → `backend/src/domain/users/uid/mod.rs` (`Uid`) + `errors.rs`.

## Interdits

- `pub enum <Struct>Error` dans le même fichier que `pub struct <Struct>` — déplacer dans `<object>/errors.rs`.
- `backend/src/<module>/<object>.rs` contenant à la fois struct et error.
- Fichiers `Errors.rs`/`Env.rs`/`Group.rs` en PascalCase — utiliser `snake_case` (`errors.rs`, `mod.rs` pour la struct) — `cargo clippy` exige minuscules.
- `backend/src/<module>/<object>/<object>.rs` — déclenche `clippy::module_inception`.

## Vérification

```bash
# snake_case et errors séparés
test -f backend/src/env/errors.rs && echo "OK env errors" || echo "FAIL"
test -f backend/src/env/mod.rs && echo "OK env mod" || echo "FAIL"
test -f backend/src/domain/groups/group/errors.rs && echo "OK group errors" || echo "FAIL"
test -f backend/src/domain/groups/group/mod.rs && echo "OK group mod" || echo "FAIL"

# pas de PascalCase, pas d'inception
! ls backend/src/env/Env.rs 2>/dev/null && echo "OK no PascalCase"
! ls backend/src/domain/groups/group/Group.rs 2>/dev/null && echo "OK no inception"

# struct sans error dans mod.rs
! rg -n "pub enum.*Error" backend/src/env/mod.rs && echo "OK Env clean"
! rg -n "pub enum.*Error" backend/src/domain/groups/group/mod.rs && echo "OK Group clean"

devenv shell -- cargo clippy -- -D warnings
devenv shell -- cargo test
```
