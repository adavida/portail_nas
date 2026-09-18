# 07 — Error Messages — English Only

> Skills de référence:
>
> - Backend: `backend/src/error.rs`, `backend/src/http/error.rs`, `backend/src/auth/*`, `backend/src/env.rs`
> - Frontend: `frontend/src/auth/oidc.ts`, `frontend/src/pages/*`, `frontend/src/env.ts`

## Objectif

Tous les messages d'erreur, `panic!`, `expect`, `tracing::warn/error`, `AppError`, `Display`/`Error` et `throw new Error` doivent être en anglais. Pas de français dans les messages destinés aux logs, aux builds ou aux réponses JSON. Les commentaires et la doc peuvent rester en français, les messages non.

## Règles

- `expect("OIDC_ISSUER_URL missing: set ...")` — pas `manquant: définis ...`
- `AppError::Internal("token exchange failed: ...")` — pas `échec d'échange`
- `throw new Error("VITE_APP_URL missing: ...")` — pas `manquant`
- `tracing::warn!("401 missing Bearer for {} {}")` — pas `manquant`
- `Display` des VOs (`UidError`, `NameError`, etc.) — déjà en anglais, garder.

## Interdits

- Français dans `panic!`, `expect`, `unwrap_or_else`, `tracing::warn/error/info` quand c'est un message d'erreur, `AppError`, `serde::de::Error::custom`, `throw new Error`.
- `rg -n "manquant|définis|échec|manquante" backend/src frontend/src` doit être vide hors commentaires.

## Vérification

```bash
rg -n "manquant|définis" backend/src frontend/src
# doit être vide

rg -n "expect\(.*manquant|panic!\(.*manquant" backend/src
# doit être vide

devenv shell -- cargo clippy -- -D warnings
devenv shell -- bash -c 'cd frontend && npx tsc --noEmit'
```
