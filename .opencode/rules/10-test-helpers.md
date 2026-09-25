# 10 — Test Helpers — Répertoire dédié, 1 helper = 1 fichier

> Règles liées: `08-struct-folder.md` (layout), `03-backend-frontend.md` (tests API), `04-workflow.md` (TDD).

## Objectif

Tout helper de test (unit ou intégration) vit dans un répertoire dédié, **1 helper = 1 fichier**. Les `mod.rs` de ces répertoires ne contiennent que `mod <helper>;` + re-exports — aucun corps de fonction.

## Chemins — sous-dossiers par type (assert, seed, app/http, env/ldap)

```text
backend/src/test_helpers/mod.rs      # #[cfg(test)] pub(crate) mod test_helpers; dans lib.rs; pub(crate) mod <type>;
backend/src/test_helpers/<type>/mod.rs      # mod <helper>; pub(crate) use <helper>::<helper>; (sauf fns privées du type, ex: inject_admin)
backend/src/test_helpers/<type>/<helper>.rs # 1 helper pub(crate) + ses fns privées annexes (ex: is_ldap_down dans seed_and_create_user.rs)
backend/tests/common/mod.rs          # pub(crate) mod <type>; — declarations seules
backend/tests/common/<type>/mod.rs   # pub(crate) mod <helper>; — pas de re-exports (les binaries appellent le chemin complet common::<type>::<helper>::<fn>)
backend/tests/common/<type>/<helper>.rs # 1 helper
```

- Types unit: `seed/` (gen_uid, gen_gid, new_user, ensure_clean, seed_and_create_user, seed_user, seed_group, cleanup), `assert/` (assert_auth_ok/fail/err), `app/` (test_app, inject_admin), `env/` (redirect_ldap_env).
- Types intégration: `seed/` (test_user, test_group, fixture, seed_user, new_group_with_member, nanos), `assert/` (uid_exists, user_groups), `http/` (send, oneshot, req, req_with_app, test_app, test_app_as_user, inject_admin, inject_user), `ldap/` (purge, redirect_env, ensure_clean).
- Frontend: `frontend/src/test/<helper>.ts`, 1 helper par fichier (`setup.ts` reste l'entrée setupFiles).

## Interdits

- `pub(crate) mod test_utils` inline dans un `mod.rs` de module applicatif (ex: `repository/ldap/users/mod.rs`) → helper dans `src/test_helpers/`.
- Helper (fn ou struct de test) défini dans un fichier `backend/tests/<ressource>.rs` hors `#[cfg(test)] mod tests` du unit → le déplacer dans `tests/common/<helper>.rs`. Dans `tests/*.rs`, ne restent que les `#[tokio::test]`.
- Plusieurs helpers dans un même fichier → un fichier par helper.
- `#![allow(dead_code)]` par fichier helper → interdit; chaque binaire de test compile tout `common`, le lint est donc structurel: un seul `#[allow(dead_code)]` sur `mod common;` en tête de chaque `tests/<ressource>.rs` le couvre tout le sous-arbre.

## Vérification

```bash
rg -n "mod test_utils" backend/src                                    # vide
rg -n "^\s*(pub )?(async )?fn |^\s*(pub )?struct " backend/src/test_helpers/mod.rs backend/tests/common/mod.rs  # vide (hors mod/use)
test -d backend/src/test_helpers && ls backend/src/test_helpers/*.rs  # 1 fichier par helper
devenv shell -- cargo clippy --all-targets -- -D warnings
devenv shell -- cargo test --features test-api
```
