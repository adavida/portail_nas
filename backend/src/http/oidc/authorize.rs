use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Response,
};
use serde::Deserialize;

use crate::{
    auth::{store, tokens},
    domain::oidc::Client,
    env::Env,
    error::AppError,
    http::oidc,
};

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

/// Authorization endpoint: client + redirect_uri verified, PKCE S256
/// stored. Existing session -> direct code (SSO), otherwise the SPA
/// login page at `{APP_URL}/login?req=...`.
pub async fn authorize(
    State(env): State<Env>,
    headers: HeaderMap,
    Query(q): Query<AuthorizeParams>,
) -> Result<Response, AppError> {
    let client_id = q.client_id.clone().unwrap_or_default();
    let redirect_uri = q.redirect_uri.clone().unwrap_or_default();
    let clients = tokens::clients(&env)?;
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
    if q.response_type.as_deref() != Some("code") {
        return Err(AppError::BadRequest("unsupported response_type".into()));
    }
    let scopes: Vec<String> = q
        .scope
        .clone()
        .unwrap_or_else(|| "openid".into())
        .split_whitespace()
        .filter(|s| *s == "openid" || client.grants(s))
        .map(String::from)
        .collect();
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
    let app_url = env.app_url.trim_end_matches('/');
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
        crate::env::Env::ensure_init();
        crate::test_helpers::app::bootstrap_clients_file(crate::env::Env::global());
        crate::auth::secrets::ensure(crate::env::Env::global()).unwrap();
        let env = crate::env::Env::global().clone();
        oidc::oidc_router(env.clone()).with_state(env)
    }

    fn env_redirect() -> String {
        std::env::var("OIDC_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:5173/callback".into())
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
        let client_id = crate::env::Env::global().oidc_client_id.clone();
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
        let env = crate::env::Env::global().clone();
        let req = Request::builder()
            .uri(format!(
                "/api/oidc/authorization?client_id={}&redirect_uri={}&response_type=code&state=state123456&code_challenge=chal&code_challenge_method=S256",
                env.oidc_client_id,
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
            location.starts_with(&format!("{}/login?req=", env.app_url.trim_end_matches('/'))),
            "anonymous authorize must send the SPA to /login?req=..., got {location}"
        );
    }

    fn urlencoding(s: &str) -> String {
        s.replace(':', "%3A")
            .replace('/', "%2F")
            .replace('?', "%3F")
    }
}
