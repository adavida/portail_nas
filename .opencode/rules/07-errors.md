# 07 — Error Messages — English Only

> Backend: `error.rs`, `http/error.rs`, `auth/*`, `env/` — Frontend: `auth/oidc.ts`, `pages/*`, `env.ts`

Tous les messages destinés aux logs, builds ou réponses JSON sont en anglais: `panic!`, `expect`, `unwrap_or_else`, `tracing::warn/error/info`, `AppError`, `Display`/`Error`, `serde::de::Error::custom`, `throw new Error`. Les commentaires et la doc restent en français.

- `expect("OIDC_ISSUER_URL missing: set ...")` — pas `manquant: définis ...`
- `AppError::Internal("token exchange failed: ...")` — pas `échec d'échange`
- `tracing::warn!("401 missing Bearer for {} {}")` — pas `manquant`
- `Display` des VOs (`UidError`, `NameError`…) — déjà en anglais, garder.

Vérif:

```bash
rg -n "manquant|définis" backend/src frontend/src        # vide hors commentaires
rg -n "expect\(.*manquant|panic!\(.*manquant" backend/src # vide
devenv shell -- cargo clippy -- -D warnings
devenv shell -- bash -c 'cd frontend && npx tsc --noEmit'
```
