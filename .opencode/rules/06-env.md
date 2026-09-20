# 06 — Environment Helpers

> Skills: backend `backend/src/env/` — frontend `frontend/src/env.ts`

## Objectif

Toutes les variables d'environnement via des helpers centralisés — un seul endroit définit les noms. Deux catégories :

- **Obligatoires** (défaut interdit): manquant/vide = erreur agrégée anglais.
- **Optionnables** (défaut autorisé): valeur personnalizable via env, défaut hard-codé documenté dans `env/mod.rs` — utilisé quand var absente ou vide.

## Backend `backend/src/env/` (seul avec `option_env!`/`env!`/`std::env::var`)

- `env/mod.rs`: struct `Env` + `static ENV: OnceLock<Env>` + `create()` (collecte toutes les vars manquantes/vides → `EnvError` agrégé anglais) + `set()` (2e appel ignoré) / `global()` / `ensure_init()` / fallback `from_compile_fallback()` (tests).
- Vars obligatoires via `take(name, ...)` → manquant = `EnvError`. Vars optionnables via `take_opt(name, default)` (`var().ok().trim().filter(!empty).unwrap_or(default)`), jamais incluses dans l'erreur.
- `env/errors.rs`: `EnvError`.
- Message type: `missing or empty env vars: A, B. set them in devenv.nix env` — pas de seuil `first-error`.
- Autres fichiers: `Env::global()` (ou `ensure_init()`) — jamais `std::env::var` direct. Pas de fallback hard-codé hors `from_compile_fallback`.
- Défauts optionnables seulement pour l'infra LDAP standard (ex: `LDAP_PEOPLE_OU`→`people`, `LDAP_GROUPS_OU`→`groups`); secrets/URLs restent obligatoires.
- Vérif: `rg -n "std::env::var|option_env!|env!" backend/src --glob '!env/**'` vide hors `#[cfg(test)]`.

## Frontend `frontend/src/env.ts` (seul avec `import.meta.env`)

- `createEnv(get?)` collecte les noms manquants → `throw` une seule `Error` (liste + hint `set ${name} in devenv.nix env (VITE_${name})`).
- `export const env`; le reste importe `env` — jamais `import.meta.env`; `vite.config.ts` valide au build.
- Vérif: `rg -n "import\.meta\.env|process\.env" frontend/src --glob '!env.ts' --glob '!vite.config.ts'` vide.

## Interdits

- `std::env::var` / `option_env!` / `env!` hors `backend/src/env/**`
- `import.meta.env` / `process.env` hors `frontend/src/env.ts` (et `vite.config.ts` build)
- Fallback hard-codé `unwrap_or("http://localhost...")` — utiliser `EnvError`/`throw` anglais (seule exception: défauts optionnables infra LDAP, cf. ci-dessus).
