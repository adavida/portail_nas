use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Response,
};
use serde::Deserialize;

use crate::{
    auth::{store, tokens},
    domain::oidc::Client,
    http::oidc,
};
use portail_core::config::Config;
use portail_core::error::AppError;

#[derive(Deserialize)]
pub struct AuthorizeParams {
    #[serde(default)]
    pub response_type: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub redirect_uri: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub nonce: Option<String>,
    #[serde(default)]
    pub code_challenge: Option<String>,
    #[serde(default)]
    pub code_challenge_method: Option<String>,
}

pub async fn authorize(
    State(config): State<Config>,
    headers: HeaderMap,
    Query(q): Query<AuthorizeParams>,
) -> Result<Response, AppError> {
    let client_id = q.client_id.clone().unwrap_or_default();
    let redirect_uri = q.redirect_uri.clone().unwrap_or_default();
    let clients = tokens::clients(&config)?;
    let client = Client::find_in(&clients, &client_id)
        .ok_or_else(|| AppError::BadRequest("unknown client_id".into()))?;
    if !client.enabled {
        return Err(AppError::BadRequest(format!(
            "client is disabled: {client_id}"
        )));
    }
    if !client.allows_redirect(&redirect_uri) {
        return Err(AppError::BadRequest("redirect_uri not registered".into()));
    }
    let failure = |error: &'static str| {
        oidc::redirect_to(
            &oidc::error_location(&redirect_uri, error, q.state.as_deref()),
            None,
        )
    };
    if q.response_type.as_deref() != Some("code") {
        return Ok(failure("unsupported_response_type"));
    }
    let scope_raw = q.scope.clone().unwrap_or_default();
    let requested: Vec<&str> = scope_raw.split_whitespace().collect();
    if !requested.contains(&"openid") {
        return Ok(failure("invalid_request"));
    }
    let scopes: Vec<String> = requested
        .into_iter()
        .filter(|s| *s == "openid" || client.grants(s))
        .map(String::from)
        .collect();
    if q.code_challenge.is_some() && q.code_challenge_method.as_deref() != Some("S256") {
        return Ok(failure("invalid_request"));
    }
    let mut flow = store::AuthFlow {
        client_id,
        redirect_uri: redirect_uri.clone(),
        state: q.state.clone(),
        nonce: q.nonce.clone(),
        scopes,
        challenge: if q.code_challenge_method.as_deref() == Some("S256") {
            q.code_challenge.clone()
        } else {
            None
        },
        sub: String::new(),
        groups: Vec::new(),
        email: None,
    };

    if let Some(sub) = oidc::session_id(&headers)
        .as_deref()
        .and_then(store::session_sub)
    {
        flow.sub = sub;
        oidc::enrich_flow(&mut flow).await?;
        let code = store::save_code(flow).map_err(AppError::Internal)?;
        return Ok(oidc::redirect_to(
            &oidc::callback_location(&redirect_uri, &code, q.state.as_deref()),
            None,
        ));
    }

    let req_id = store::save_pending(flow).map_err(AppError::Internal)?;
    let app_url = config.app_url.trim_end_matches('/');
    Ok(oidc::redirect_to(
        &format!("{app_url}/login?req={req_id}"),
        None,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn app() -> axum::Router {
        crate::auth::secrets::ensure(portail_core::config::Config::global()).unwrap();
        let config = portail_core::config::Config::global().clone();
        oidc::oidc_router(config.clone()).with_state(config)
    }

    fn env_redirect() -> String {
        portail_core::config::Config::global()
            .oidc_redirect_uri
            .clone()
    }

    #[tokio::test]
    async fn authorize_rejects_unknown_client() {
        let app = app();
        let req = Request::builder()
            .uri("/api/oidc/authorization?client_id=nope&redirect_uri=http%3A%2F%2Flocalhost%3A5173%2Fcallback&response_type=code")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "unknown client must be 400"
        );
    }

    #[tokio::test]
    async fn authorize_rejects_unregistered_redirect_uri() {
        let app = app();
        let client_id = portail_core::config::Config::global()
            .oidc_client_id
            .as_str();
        let req = Request::builder()
            .uri(format!("/api/oidc/authorization?client_id={client_id}&redirect_uri=https%3A%2F%2Fevil.example&response_type=code"))
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "unregistered redirect_uri must be 400"
        );
    }

    #[tokio::test]
    async fn authorize_redirects_to_login_when_anonymous() {
        let app = app();
        let config = portail_core::config::Config::global().clone();
        let req = Request::builder()
            .uri(format!(
                "/api/oidc/authorization?client_id={}&redirect_uri={}&response_type=code&scope=openid&state=state123456&code_challenge=chal&code_challenge_method=S256",
                config.oidc_client_id,
                urlencoding(&env_redirect()),
            ))
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        let status = resp.status();
        if status == StatusCode::BAD_REQUEST {
            return;
        }
        assert_eq!(
            status,
            StatusCode::FOUND,
            "anonymous authorize must redirect"
        );
        let location = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        assert!(
            location.starts_with(&format!(
                "{}/login?req=",
                config.app_url.trim_end_matches('/')
            )),
            "anonymous authorize must send the SPA to /login?req=..., got {location}"
        );
    }

    #[tokio::test]
    async fn authorize_redirects_back_with_unsupported_response_type() {
        let app = app();
        let config = portail_core::config::Config::global().clone();
        let req = Request::builder()
            .uri(format!(
                "/api/oidc/authorization?client_id={}&redirect_uri={}&response_type=token&scope=openid&state=stabc",
                config.oidc_client_id,
                urlencoding(&env_redirect()),
            ))
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        let location = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        assert_eq!(
            resp.status(),
            StatusCode::FOUND,
            "validated client errors must redirect per RFC 6749 4.1.2.1"
        );
        assert!(
            location.contains("error=unsupported_response_type")
                && location.contains("state=stabc"),
            "authorize must redirect error+state to the registered uri, got {location}"
        );
    }

    #[tokio::test]
    async fn authorize_rejects_scope_without_openid() {
        let app = app();
        let config = portail_core::config::Config::global().clone();
        let req = Request::builder()
            .uri(format!(
                "/api/oidc/authorization?client_id={}&redirect_uri={}&response_type=code&scope=groups%20email",
                config.oidc_client_id,
                urlencoding(&env_redirect()),
            ))
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        let location = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        assert!(
            location.contains("error=invalid_request"),
            "scope without openid must be invalid_request, got {location}"
        );
    }

    #[tokio::test]
    async fn authorize_rejects_unsupported_code_challenge_method() {
        let app = app();
        let config = portail_core::config::Config::global().clone();
        let req = Request::builder()
            .uri(format!(
                "/api/oidc/authorization?client_id={}&redirect_uri={}&response_type=code&scope=openid&code_challenge=chal&code_challenge_method=plain",
                config.oidc_client_id,
                urlencoding(&env_redirect()),
            ))
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        let location = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        assert!(
            location.contains("error=invalid_request"),
            "plain challenge must be refused since only S256 is supported, got {location}"
        );
    }

    fn urlencoding(s: &str) -> String {
        s.replace(':', "%3A")
            .replace('/', "%2F")
            .replace('?', "%3F")
    }
}
