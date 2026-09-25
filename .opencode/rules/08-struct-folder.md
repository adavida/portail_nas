# 08 — Struct Folder

> Skills de référence:
>
> - Structure: `.opencode/skills/backend-structure/SKILL.md` (trigger `backend/src/`)
> - VO: `.opencode/rules/05-value-objects.md`

## Objectif

1 objet = 1 dossier `<object>/` : `mod.rs` (struct) + `errors.rs` (snake_case, error) — zéro `clippy::module_inception`, zéro `#[allow(module_inception)]`.

1 fonction publique libre = 1 fichier dans le dossier du module, `mod.rs` ne contenant que `mod`/`pub use` + helpers partagés `pub(crate)`. Les helpers de test ne vivent **pas** dans le `mod.rs` → `src/test_helpers/` (règle `10-test-helpers.md`).

## Règle

```text
backend/src/<module>/<object>/mod.rs     # pub struct <Struct> + impl try_new/create/from_* + #[cfg(test)] — pas de Error dedans
backend/src/<module>/<object>/errors.rs  # pub enum <Struct>Error + Display + Error
backend/src/<module>/<object>/mod.rs     # pub mod errors; pub use errors::<Struct>Error;
backend/src/<module>/mod.rs?:            # pub mod <object>; pub use <object>::{<Struct>, <Struct>Error}; — plat
```

- Racine type `env.rs` ⇒ `env/mod.rs` (struct) + `env/errors.rs`.
- Frontend identique: `frontend/src/<module>/<object>/errors.ts`.

### Fonctions publiques libres (ex: `repository/ldap/users/`)

```text
backend/src/<module>/<group>/mod.rs              # mod <fn>; pub use <fn>::<fn>; + helpers pub(crate) + #[cfg(test)] pub(crate) mod test_helpers
backend/src/<module>/<group>/<fn>.rs             # 1 pub async fn <fn> + ses fns privées annexes + #[cfg(test)] mod tests collocalisé
```

Exemple standard: `repository/ldap/users/` (`list_users.rs`, `create_user.rs`, `update_user.rs`, …).

Helper de test → `src/test_helpers/` (1 helper = 1 fichier, règle `10-test-helpers.md`), jamais de `test_utils` inline dans le `mod.rs`.

Exemples standards: `env/` (`Env`/`EnvError`), `domain/groups/group/` (`Group`/`GroupError`), `domain/groups/members/` (`Members`/`MembersError`), VOs `users/uid/` (`Uid`/`UidError`).

## Interdits

- `pub enum <Struct>Error` dans le même fichier que `pub struct <Struct>` → toujours `errors.rs`.
- Plusieurs `pub fn`/`pub async fn` libres dans un même fichier (hors `impl`) → 1 fonction = 1 fichier ; `mod.rs` = déclarations + `pub use` + helpers `pub(crate)` uniquement.
- `backend/src/<module>/<object>/<object>.rs` (même nom que le dossier) → `clippy::module_inception` — si le lint se déclenche, **renommer** (struct → `mod.rs`, error → `errors.rs`), jamais `#[allow(clippy::module_inception)]`. Exception unique: `#[allow(...)]` avec `// ponytail: ...` si le nommage redondant est réellement voulu.
- Fichiers PascalCase (`Env.rs`, `Group.rs`) → clippy exige minuscules.
- Sub-module même nom que le parent qui déclenche E0255 → ne **jamais** contourner avec `mod.rs` + `#[path = "Env.rs"] pub mod env_;` : réécrire en `mod.rs` pour la structure + `errors.rs` pour l'error.

## Vérification

```bash
# snake_case + errors séparés + struct sans error
test -f backend/src/env/mod.rs && test ! -f backend/src/env/Env.rs && echo "OK env"
rg -n "pub enum.*Error" backend/src --glob '*/mod.rs' # doit être vide (hors sub-modules)
rg -n "allow\(clippy::module_inception\)" backend/src # doit être vide

devenv shell -- cargo clippy -- -D warnings
devenv shell -- cargo test
```
