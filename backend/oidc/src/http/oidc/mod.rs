use axum::{
    Router,
    body::Body,
    http::{HeaderMap, StatusCode, header},
    response::Response,
    routing::{get, post},
};

use portail_core::config::Config;
use portail_core::error::AppError;

pub mod authorize;
pub mod discovery;
pub mod jwks;
pub mod login;
pub mod logout;
pub mod token;
pub mod userinfo;

pub fn oidc_router(config: Config) -> Router<Config> {
    Router::new()
        .route(
            "/.well-known/openid-configuration",
            get(discovery::discovery),
        )
        .route("/jwks.json", get(jwks::jwks))
        .route("/logout", get(logout::logout))
        .route("/api/oidc/authorization", get(authorize::authorize))
        .route("/api/oidc/login", post(login::login))
        .route("/api/oidc/token", post(token::token))
        .route("/api/oidc/userinfo", get(userinfo::userinfo))
        .with_state(config)
}

pub(crate) const SESSION_COOKIE: &str = "portail_session";

pub(crate) fn session_id(headers: &HeaderMap) -> Option<String> {
    let cookie = headers.get("cookie")?.to_str().ok()?;
    cookie
        .split(';')
        .find_map(|pair| pair.trim().strip_prefix(&format!("{SESSION_COOKIE}=")))
        .map(|sid| sid.trim().to_string())
}

pub(crate) fn session_cookie(config: &Config, sid: &str) -> String {
    let secure = if config.oidc_issuer_url.starts_with("https://") {
        "; Secure"
    } else {
        ""
    };
    format!("{SESSION_COOKIE}={sid}; Path=/; HttpOnly; SameSite=Lax{secure}")
}

pub(crate) fn expired_cookie() -> String {
    format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

pub(crate) fn callback_location(redirect_uri: &str, code: &str, state: Option<&str>) -> String {
    match state {
        Some(state) => format!(
            "{redirect_uri}?code={}&state={}",
            encode_query_value(code),
            encode_query_value(state)
        ),
        None => format!("{redirect_uri}?code={}", encode_query_value(code)),
    }
}

pub(crate) fn encode_query_value(raw: &str) -> String {
    let mut encoded = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

pub(crate) fn redirect_to(location: &str, cookie: Option<String>) -> Response {
    let mut builder = Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, location);
    if let Some(cookie) = cookie {
        builder = builder.header(header::SET_COOKIE, cookie);
    }
    builder
        .body(Body::empty())
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

pub(crate) async fn enrich_flow(flow: &mut crate::auth::store::AuthFlow) -> Result<(), AppError> {
    let user = portail_directory::controllers::users::list()
        .await?
        .into_iter()
        .find(|u| u.uid.as_str() == flow.sub);
    let Some(user) = user else {
        return Ok(());
    };
    if flow.scopes.iter().any(|s| s == "groups") {
        flow.groups = user.groups.iter().map(|g| g.as_str().to_string()).collect();
    }
    if flow.scopes.iter().any(|s| s == "email") && !user.email.as_str().is_empty() {
        flow.email = Some(user.email.as_str().to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn callback_location_encodes_state() {
        let location =
            callback_location("http://localhost:5173/callback", "abc123", Some("a&b=#c"));

        assert_eq!(
            location, "http://localhost:5173/callback?code=abc123&state=a%26b%3D%23c",
            "state metacharacters must be percent-encoded"
        );
    }

    #[test]
    fn session_id_extracts_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            HeaderValue::from_static("other=1; portail_session=abc123"),
        );

        let sid = session_id(&headers);

        assert_eq!(
            sid.as_deref(),
            Some("abc123"),
            "session sid should be parsed"
        );
    }

    #[test]
    fn session_id_absent_without_cookie() {
        let headers = HeaderMap::new();

        let sid = session_id(&headers);

        assert_eq!(sid, None, "no cookie header means no session");
    }

    #[test]
    fn session_cookie_secure_only_on_https_issuer() {
        let mut config = portail_core::config::Config::global().clone();
        config.oidc_issuer_url = "https://portail.example.com".into();

        let secure = session_cookie(&config, "sid");
        config.oidc_issuer_url = "http://127.0.0.1:3000".into();
        let plain = session_cookie(&config, "sid");

        assert!(
            secure.contains("Secure"),
            "https issuer must set Secure: {secure}"
        );
        assert!(
            !plain.contains("Secure"),
            "http issuer must not set Secure: {plain}"
        );
        assert!(
            secure.contains("HttpOnly") && secure.contains("SameSite=Lax"),
            "session cookie must be HttpOnly SameSite=Lax"
        );
    }
}
