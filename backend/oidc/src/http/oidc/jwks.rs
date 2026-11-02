use axum::Json;
use serde::Serialize;

use crate::auth::key;
use portail_core::error::AppError;

#[derive(Serialize)]
pub struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Serialize)]
pub struct Jwk {
    kty: &'static str,
    #[serde(rename = "use")]
    public_key_use: &'static str,
    alg: &'static str,
    kid: &'static str,
    n: &'static str,
    e: &'static str,
}

pub async fn jwks() -> Result<Json<Jwks>, AppError> {
    let key = key::global().ok_or_else(|| {
        AppError::Internal("signing key not available: generate it in the OIDC secrets dir".into())
    })?;
    Ok(Json(Jwks {
        keys: vec![Jwk {
            kty: "RSA",
            public_key_use: "sig",
            alg: "RS256",
            kid: &key.kid,
            n: &key.n,
            e: &key.e,
        }],
    }))
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

        let key = &v.keys[0];
        assert_eq!(key.kty, "RSA", "jwk kty must be RSA");
        assert_eq!(key.alg, "RS256", "jwk alg must be RS256");
        assert!(!key.n.is_empty(), "jwk modulus must be non-empty");
        assert!(!key.e.is_empty(), "jwk exponent must be non-empty");
    }
}
