use axum::Json;
use serde_json::json;

use crate::auth::key;
use portail_core::error::AppError;

/// JWKS — the provider public key, exposed for verification by
/// third-party clients (`{issuer}/jwks.json`).
pub async fn jwks() -> Result<Json<serde_json::Value>, AppError> {
    let key = key::global().ok_or_else(|| {
        AppError::Internal("signing key not available: generate it in the OIDC secrets dir".into())
    })?;
    Ok(Json(json!({
        "keys": [{
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": key.kid,
            "n": key.n,
            "e": key.e,
        }],
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn jwks_returns_rsa_key() {
        let result = super::jwks().await;
        let Ok(Json(v)) = result else {
            return;
        };

        assert_eq!(v["keys"][0]["kty"], "RSA", "jwk kty must be RSA");
        assert_eq!(v["keys"][0]["alg"], "RS256", "jwk alg must be RS256");
        assert!(
            !v["keys"][0]["n"].as_str().unwrap_or("").is_empty(),
            "jwk modulus must be non-empty"
        );
        assert!(
            !v["keys"][0]["e"].as_str().unwrap_or("").is_empty(),
            "jwk exponent must be non-empty"
        );
    }
}
