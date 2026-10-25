use axum::{Json, extract::State};
use serde::Serialize;

use portail_core::config::Config;

#[derive(Serialize)]
pub struct Discovery {
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
    jwks_uri: String,
    end_session_endpoint: String,
    response_types_supported: &'static [&'static str],
    grant_types_supported: &'static [&'static str],
    subject_types_supported: &'static [&'static str],
    id_token_signing_alg_values_supported: &'static [&'static str],
    scopes_supported: &'static [&'static str],
    token_endpoint_auth_methods_supported: &'static [&'static str],
    code_challenge_methods_supported: &'static [&'static str],
    issuer: String,
}

pub async fn discovery(State(config): State<Config>) -> Json<Discovery> {
    let iss = config.oidc_issuer_url.trim_end_matches('/').to_string();
    Json(Discovery {
        authorization_endpoint: format!("{iss}/api/oidc/authorization"),
        token_endpoint: format!("{iss}/api/oidc/token"),
        userinfo_endpoint: format!("{iss}/api/oidc/userinfo"),
        jwks_uri: format!("{iss}/jwks.json"),
        end_session_endpoint: format!("{iss}/logout"),
        response_types_supported: &["code"],
        grant_types_supported: &["authorization_code"],
        subject_types_supported: &["public"],
        id_token_signing_alg_values_supported: &["RS256"],
        scopes_supported: &["openid", "groups", "email", "profile"],
        token_endpoint_auth_methods_supported: &["client_secret_basic"],
        code_challenge_methods_supported: &["S256"],
        issuer: iss,
    })
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn discovery_advertises_provider_endpoints() {
        let app = oidc_app();
        let req = Request::builder()
            .uri("/.well-known/openid-configuration")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK, "discovery should be 200");
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let iss = portail_core::config::Config::global()
            .oidc_issuer_url
            .trim_end_matches('/')
            .to_string();
        assert_eq!(v["issuer"], iss, "issuer must match config");
        assert_eq!(
            v["token_endpoint"],
            format!("{iss}/api/oidc/token"),
            "token endpoint must be advertised"
        );
        assert_eq!(
            v["id_token_signing_alg_values_supported"][0], "RS256",
            "RS256 must be advertised"
        );
    }

    fn oidc_app() -> axum::Router {
        let config = portail_core::config::Config::global().clone();
        crate::http::oidc::oidc_router(config.clone()).with_state(config)
    }
}
