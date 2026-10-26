use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Response,
};
use serde::Deserialize;

use crate::{auth::store, http::oidc};
use portail_core::config::Config;

#[derive(Deserialize)]
pub struct LogoutQuery {
    #[serde(default)]
    pub rd: Option<String>,
}

pub async fn logout(
    State(config): State<Config>,
    headers: HeaderMap,
    Query(q): Query<LogoutQuery>,
) -> Response {
    if let Some(sid) = oidc::session_id(&headers) {
        store::destroy_session(&sid);
    }
    let root = config.app_url.trim_end_matches('/');
    let target =
        q.rd.filter(|rd| *rd == root || rd.starts_with(&format!("{root}/")))
            .unwrap_or_else(|| format!("{root}/"));
    oidc::redirect_to(&target, Some(oidc::expired_cookie()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn app() -> axum::Router {
        let config = portail_core::config::Config::global().clone();
        oidc::oidc_router(config.clone()).with_state(config)
    }

    #[tokio::test]
    async fn logout_redirects_to_app_url_by_default() {
        let app = app();
        let req = Request::builder()
            .uri("/logout")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::FOUND, "logout must redirect");
        let location = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let app_url = portail_core::config::Config::global().app_url.as_str();
        assert!(
            location.starts_with(app_url),
            "default redirect should be the portal: {location}"
        );
    }

    #[tokio::test]
    async fn logout_rejects_foreign_rd() {
        let app = app();
        let req = Request::builder()
            .uri("/logout?rd=https://evil.example.com/")
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
            !location.contains("evil"),
            "foreign rd must be ignored, got {location}"
        );
    }

    #[tokio::test]
    async fn logout_rejects_lookalike_host_rd() {
        let app = app();
        let app_url = portail_core::config::Config::global()
            .app_url
            .trim_end_matches('/')
            .to_string();
        let req = Request::builder()
            .uri(format!("/logout?rd={app_url}.evil.example"))
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
            location,
            format!("{app_url}/"),
            "lookalike host rd must fall back to the portal root"
        );
    }

    #[tokio::test]
    async fn logout_clears_session_cookie() {
        let app = app();
        let req = Request::builder()
            .uri("/logout")
            .header("cookie", "portail_session=abc")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        let cookie = resp
            .headers()
            .get("set-cookie")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        assert!(
            cookie.contains("portail_session=;") && cookie.contains("Max-Age=0"),
            "cookie must be cleared, got {cookie}"
        );
    }
}
