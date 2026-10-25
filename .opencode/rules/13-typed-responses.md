# 13 — Response JSON — Structs `Serialize`, pas de `json!` en prod

> Règles liées: `05-value-objects.md` (serde transparent), `08-struct-folder.md` (layout), `07-errors.md` (messages anglais).

## Objectif

Tout corps JSON **de sortie** (réponse HTTP de prod) est produit par un `struct`/`enum` `#[derive(Serialize)]`, jamais par la macro `json!` ni par un `Json<serde_json::Value>`. Les clés et l'ordre du JSON sont définis par l'ordre des champs du struct.

## Pattern

Le struct vit dans le fichier du handler qui le renvoie (précédents: `AuthConfig` dans `http/auth/config.rs`, `TokenResponse` dans `http/oidc/token.rs`, `Discovery`/`Jwks`/`Userinfo`/`Login` dans `http/oidc/`, `ApiError` dans `core/src/error.rs`):

```rust
#[derive(Serialize)]
pub struct Login {
    redirect: String,
}

pub async fn login(...) -> Result<Json<Login>, AppError> {
    Ok(Json(Login { redirect }))
}
```

- Clés JSON non-identifiants Rust (`"use"` dans un JWK): champ renommé descriptif + `#[serde(rename = "use")]`.
- Listes statiques: `&'static [&'static str]`; valeurs déjà typées du domaine (`Claims`, VOs): champs directs, pas de conversion.
- Erreurs: un seul `ApiError { error: String }` dans `core/src/error.rs`, construit par `IntoResponse for AppError` — jamais de `json!({"error": …})` dispersés.
- Partage entre handlers: un struct de réponse peut être réutilisé par plusieurs handlers du même contrat (ex: `Userinfo` pour `/userinfo` et `/me`); des contrats distincts gardent des structs distincts.

## Tests

Les tests (`#[cfg(test)]`, `backend/server/tests/`) gardent `json!` pour les **entrées** (payloads requis) et la **lecture** des réponses (`serde_json::Value`, assertions par clé) — c'est la convention des règles 03/10, le struct de sortie n'impose rien au test.

## Interdits

- `Json<serde_json::Value>` dans un handler de prod.
- `json!(…)` hors `#[cfg(test)]` / `backend/server/tests/` / `src/test_helpers/`.
- `serde_json::Value` comme type de champ dans un struct de sortie ou un VO sérialisé (ex: `Claims.aud` est `String`, pas `Value`).

## Vérification

```bash
rg -n "json!\(|Json<serde_json::Value>" backend/*/src --glob '*/src/**/*.rs' \
  | grep -v "cfg(test)"   # vide hors blocs #[cfg(test)] (vérifier à la main si besoin)
rg -n "serde_json::Value" backend/*/src | grep -v "cfg(test)"   # vide
devenv shell -- cargo clippy --workspace --all-targets --features test-api -- -D warnings
devenv shell -- cargo test --features test-api
```
