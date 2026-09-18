# 06 — Environment Helpers

> Skills de référence:
>
> - Backend: `backend/src/env.rs` (helper centralisé)
> - Frontend: `frontend/src/env.ts` (helper centralisé)

## Objectif

Grouper toutes les variables d'environnement via des helpers centralisés. Plus d'accès dispersé `std::env::var`, `option_env!`, `env!` ou `import.meta.env` hors des helpers. Un seul endroit définit les noms, les valeurs par défaut (aucun — panic si manquant) et les messages d'erreur.

## Backend `backend/src/env.rs`

```rust
// backend/src/env.rs — single source of truth
pub const OIDC_ISSUER_URL: &str = option_env!("OIDC_ISSUER_URL").expect("OIDC_ISSUER_URL missing: set OIDC_ISSUER_URL in devenv.nix env");
pub const OIDC_CLIENT_ID: &str = option_env!("OIDC_CLIENT_ID").expect("OIDC_CLIENT_ID missing: set OIDC_CLIENT_ID in devenv.nix env");
pub const OIDC_CLIENT_SECRET: &str = option_env!("OIDC_CLIENT_SECRET").expect("OIDC_CLIENT_SECRET missing: set OIDC_CLIENT_SECRET in devenv.nix env");
pub const OIDC_REDIRECT_URI: &str = option_env!("OIDC_REDIRECT_URI").expect("OIDC_REDIRECT_URI missing: set OIDC_REDIRECT_URI in devenv.nix env");
pub const APP_URL: &str = option_env!("APP_URL").expect("APP_URL missing: set APP_URL in devenv.nix env");
pub const BIND_ADDR: &str = option_env!("BIND_ADDR").expect("BIND_ADDR missing: set BIND_ADDR in devenv.nix env");

pub fn oidc_issuer_url() -> String { OIDC_ISSUER_URL.to_string() }
// ... idem pour chaque var
```

Règles:

- `backend/src/env.rs` est le seul fichier autorisé à contenir `option_env!`, `env!`, `std::env::var`.
- Tous les autres fichiers (`auth/*`, `http/*`, `controllers/*`, `repository/*`, `main.rs`, `tests/*`) importent `crate::env::{OIDC_ISSUER_URL, ...}` ou `crate::env::oidc_issuer_url()` — jamais `std::env::var` direct.
- Pas de `unwrap_or("http://...")` avec URL en dur — un var manquant doit `panic!` au build (const) ou `expect` au runtime avec message anglais.

Vérification:

```bash
rg -n "std::env::var|option_env!|env!" backend/src --glob '!env.rs' --glob '!main.rs'
# doit être vide hors `backend/src/env.rs` et `#[cfg(test)]`
```

## Frontend `frontend/src/env.ts`

```ts
// frontend/src/env.ts — single source of truth
function requireEnv(name: string, value: string | undefined): string {
  if (!value)
    throw new Error(
      `${name} missing: set ${name} in devenv.nix env (VITE_${name})`,
    );
  return value;
}
export const APP_URL = requireEnv(
  "APP_URL",
  import.meta.env.VITE_APP_URL as string | undefined,
);
export const OIDC_ISSUER_URL = requireEnv(
  "OIDC_ISSUER_URL",
  import.meta.env.VITE_OIDC_ISSUER_URL as string | undefined,
);
// ...
export function appUrl(): string {
  return APP_URL;
}
```

Règles:

- `frontend/src/env.ts` est le seul fichier autorisé à lire `import.meta.env` / `process.env`.
- Tous les autres fichiers (`App.tsx`, `auth/oidc.ts`, `pages/*`, `components/*`) importent `from "../env"` — jamais `import.meta.env` direct.

Vérification:

```bash
rg -n "import\.meta\.env|process\.env" frontend/src --glob '!env.ts' --glob '!vite.config.ts'
# doit être vide hors `frontend/src/env.ts` et `vite.config.ts` (qui valide au build)
```

## Interdits

- `std::env::var` / `option_env!` / `env!` hors `backend/src/env.rs`
- `import.meta.env` / `process.env` hors `frontend/src/env.ts` (et `vite.config.ts` pour la validation build)
- Fallback hard-codé `unwrap_or("http://localhost...")` — utiliser `expect` avec message anglais

## Vérification globale

```bash
rg -n "std::env::var|option_env!|env!" backend/src --glob '!env.rs' | grep -v "#\[cfg(test)\]"
rg -n "import\.meta\.env" frontend/src --glob '!env.ts'
```
