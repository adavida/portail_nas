use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode_header};
use serde::Deserialize;

use crate::{auth::Claims, env::Env};

#[derive(Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}
#[derive(Deserialize)]
struct Jwk {
    n: String,
    e: String,
    kid: Option<String>,
    // alg returned by Authelia JWKS (RS256), unverified — Validation fixed to RS256
    #[serde(default, rename = "alg")]
    _alg: Option<String>,
}

async fn fetch_jwks(env: &Env) -> Result<Jwks, String> {
    let iss = &env.oidc_issuer_url;
    let url = format!("{iss}/jwks.json");
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;
    let jwks = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Jwks>()
        .await
        .map_err(|e| e.to_string())?;
    Ok(jwks)
}

async fn fetch_userinfo(env: &Env, token: &str) -> Result<Claims, String> {
    let url = format!("{}/api/oidc/userinfo", env.oidc_issuer_url);
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("userinfo failed: {}", resp.status()));
    }
    let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let sub = v
        .get("sub")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();
    let groups = v
        .get("groups")
        .and_then(|g| g.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let email = v
        .get("email")
        .and_then(|e| e.as_str())
        .map(|s| s.to_string());
    let preferred_username = v
        .get("preferred_username")
        .and_then(|p| p.as_str())
        .map(|s| s.to_string());
    Ok(Claims {
        sub,
        aud: serde_json::json!([env.oidc_client_id.clone()]),
        iss: env.oidc_issuer_url.clone(),
        exp: 9999999999,
        groups,
        email,
        preferred_username,
    })
}

pub async fn verify_token(env: &Env, token: &str) -> Result<Claims, String> {
    // Try JWT first (id_token)
    if let Ok(header) = decode_header(token) {
        if let Ok(jwks) = fetch_jwks(env).await {
            let kid = header.kid.clone();
            let jwk = if let Some(kid) = kid {
                jwks.keys
                    .iter()
                    .find(|k| k.kid.as_deref() == Some(&kid))
                    .or(jwks.keys.first())
            } else {
                jwks.keys.first()
            };
            if let Some(jwk) = jwk {
                let mut validation = Validation::new(Algorithm::RS256);
                validation.set_audience(&[&env.oidc_client_id]);
                validation.set_issuer(&[&env.oidc_issuer_url]);
                if let Ok(key) = DecodingKey::from_rsa_components(&jwk.n, &jwk.e) {
                    if let Ok(data) = jsonwebtoken::decode::<Claims>(token, &key, &validation) {
                        // if the JWT already contains groups, use it as-is, otherwise enrich via userinfo
                        if !data.claims.groups.is_empty() {
                            return Ok(data.claims);
                        }
                        if let Ok(uinfo) = fetch_userinfo(env, token).await {
                            if !uinfo.groups.is_empty() {
                                return Ok(uinfo);
                            }
                        }
                        return Ok(data.claims);
                    }
                }
            }
        }
    }
    // Fallback opaque access_token via userinfo
    fetch_userinfo(env, token).await
}

#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

pub async fn require_auth(
    State(env): State<Env>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if req.extensions().get::<AuthUser>().is_some() {
        tracing::info!("auth bypass via injected AuthUser for {}", req.uri());
        return Ok(next.run(req).await);
    }
    let auth = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("");
    if token.is_empty() {
        tracing::warn!("401 missing Bearer for {} {}", req.method(), req.uri());
        return Err(StatusCode::UNAUTHORIZED);
    }
    tracing::info!("verifying token for {} {}", req.method(), req.uri());
    match verify_token(&env, token).await {
        Ok(claims) => {
            tracing::info!(
                "auth ok sub={} groups={:?} for {}",
                claims.sub,
                claims.groups,
                req.uri()
            );
            req.extensions_mut().insert(AuthUser(claims));
            Ok(next.run(req).await)
        }
        Err(e) => {
            tracing::warn!(
                "401 token invalid for {} {}: {}",
                req.method(),
                req.uri(),
                e
            );
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

pub async fn require_admin(req: Request, next: Next) -> Result<Response, StatusCode> {
    let Some(AuthUser(claims)) = req.extensions().get::<AuthUser>().cloned() else {
        tracing::warn!("403 no AuthUser for {}", req.uri());
        return Err(StatusCode::FORBIDDEN);
    };
    if claims.is_admin() {
        Ok(next.run(req).await)
    } else {
        tracing::warn!(
            "403 forbidden for sub={} groups={:?} on {}",
            claims.sub,
            claims.groups,
            req.uri()
        );
        Err(StatusCode::FORBIDDEN)
    }
}
