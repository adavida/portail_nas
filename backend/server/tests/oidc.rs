use axum::http::{Method, StatusCode};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::json;
use sha2::{Digest, Sha256};

#[allow(dead_code)]
mod common;
use common::{
    http::send_raw::send_raw,
    seed::{oidc_client_secret::oidc_client_secret, test_user::TestUser},
};

const VERIFIER: &str = "portail-test-pkce-verifier-0123456789-abcdefghij";

fn challenge() -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(VERIFIER.as_bytes()))
}

fn auth_url(client_id: &str, redirect_uri: &str, state: &str) -> String {
    format!(
        "/api/oidc/authorization?client_id={}&redirect_uri={}&response_type=code&state={state}&scope=openid%20groups%20email%20profile&code_challenge={}&code_challenge_method=S256",
        client_id,
        urlencoding(redirect_uri),
        urlencoding(&challenge()),
    )
}

fn urlencoding(s: &str) -> String {
    s.replace('%', "%25")
        .replace(':', "%3A")
        .replace('/', "%2F")
        .replace('?', "%3F")
        .replace('&', "%26")
        .replace('=', "%3D")
}

fn header_pairs(pairs: &[(&'static str, String)]) -> Vec<(&'static str, String)> {
    pairs.to_vec()
}

fn session_cookie(set_cookie: Option<&axum::http::HeaderValue>) -> String {
    set_cookie
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .split(';')
        .next()
        .unwrap_or_default()
        .to_string()
}

fn req_id_from(location: &str) -> String {
    location
        .split("req=")
        .nth(1)
        .unwrap_or_default()
        .split('&')
        .next()
        .unwrap_or_default()
        .to_string()
}

async fn provider_available() -> bool {
    let (status, _, _, _) = send_raw(Method::GET, "/jwks.json", &[], None).await;
    status == StatusCode::OK
}

#[tokio::test]
async fn discovery_advertises_endpoints() {
    let (status, _, body, _) =
        send_raw(Method::GET, "/.well-known/openid-configuration", &[], None).await;

    assert_eq!(status, StatusCode::OK, "discovery must be 200");
    let iss = portail_core::config::Config::global()
        .oidc_issuer_url
        .as_str()
        .trim_end_matches('/')
        .to_string();

    assert_eq!(body["issuer"], iss, "issuer should match config");
    assert_eq!(
        body["authorization_endpoint"],
        format!("{iss}/api/oidc/authorization"),
        "authorization endpoint advertised"
    );
}

#[tokio::test]
async fn full_authorization_code_flow_issues_verified_tokens() {
    let user = TestUser::new("apiofidc");
    let (status, _) = user.create().await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }
    assert_eq!(status, StatusCode::CREATED);
    if !provider_available().await {
        return;
    }
    let redirect_uri = portail_core::config::Config::global()
        .oidc_redirect_uri
        .as_str();
    let state = "state1234567890";

    let url = auth_url(
        portail_core::config::Config::global()
            .oidc_client_id
            .as_str(),
        redirect_uri,
        state,
    );
    let (status, headers, _, _) = send_raw(Method::GET, &url, &[], None).await;
    assert_eq!(
        status,
        StatusCode::FOUND,
        "authorize must redirect to login"
    );
    let location = headers
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        location.starts_with(&format!(
            "{}/login?req=",
            portail_core::config::Config::global()
                .app_url
                .as_str()
                .trim_end_matches('/')
        )),
        "should redirect to the SPA login route: {location}"
    );
    let req_id = req_id_from(&location);

    let (status, headers, body, _) = send_raw(
        Method::POST,
        "/api/oidc/login",
        &header_pairs(&[("content-type", "application/json".to_string())]),
        Some(
            json!({
                "req": req_id,
                "username": user.uid,
                "password": user.password,
            })
            .to_string(),
        ),
    )
    .await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }
    assert_eq!(status, StatusCode::OK, "valid login must return 200 JSON");
    let cookie = session_cookie(headers.get("set-cookie"));
    assert!(!cookie.is_empty(), "login must set portail_session cookie");
    let redirect = body["redirect"].as_str().unwrap_or_default().to_string();
    assert!(
        redirect.starts_with(redirect_uri),
        "redirect must target the registered redirect_uri: {redirect}"
    );
    let code = redirect
        .split("code=")
        .nth(1)
        .unwrap_or_default()
        .split('&')
        .next()
        .unwrap_or_default()
        .to_string();
    assert!(
        redirect.contains(&format!("state={state}")),
        "state must be echoed back: {redirect}"
    );

    let Some((client_id, secret)) = oidc_client_secret() else {
        return;
    };
    let basic = base64::engine::general_purpose::STANDARD.encode(format!("{client_id}:{secret}"));
    let (status, _, body, _) = send_raw(
        Method::POST,
        "/api/oidc/token",
        &header_pairs(&[
            (
                "content-type",
                "application/x-www-form-urlencoded".to_string(),
            ),
            ("authorization", format!("Basic {basic}")),
        ]),
        Some(format!(
            "grant_type=authorization_code&code={}&redirect_uri={}&code_verifier={}",
            urlencoding(&code),
            urlencoding(redirect_uri),
            urlencoding(VERIFIER),
        )),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "token exchange should be 200");
    let access = body["access_token"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(!access.is_empty(), "access_token must be issued");

    let (status, headers, _, _) = send_raw(
        Method::GET,
        &auth_url(
            portail_core::config::Config::global()
                .oidc_client_id
                .as_str(),
            redirect_uri,
            "state2",
        ),
        &header_pairs(&[("cookie", cookie.clone())]),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FOUND,
        "SSO authorize should succeed with session cookie"
    );
    let sso_location = headers
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        sso_location.starts_with(redirect_uri) && sso_location.contains("code="),
        "SSO must return code directly to redirect_uri: {sso_location}"
    );

    let (status, _, body, _) = send_raw(
        Method::GET,
        "/api/oidc/userinfo",
        &header_pairs(&[("authorization", format!("Bearer {access}"))]),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "userinfo must accept our token");
    assert_eq!(
        body["sub"], user.uid,
        "userinfo sub must match the authenticated uid"
    );
    assert!(
        body["groups"].as_array().unwrap().contains(&json!("user")),
        "seeded user should belong to group 'user', got {:?}",
        body["groups"]
    );

    let plain = tower::ServiceExt::oneshot(
        portail_server::app(),
        axum::http::Request::builder()
            .method("GET")
            .uri("/api/users")
            .header("authorization", format!("Bearer {access}"))
            .body(axum::body::Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(
        plain.status(),
        StatusCode::FORBIDDEN,
        "token must verify (403 non-admin, not 401)"
    );

    let (status, headers, _, _) = send_raw(
        Method::GET,
        "/logout",
        &header_pairs(&[("cookie", cookie.clone())]),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FOUND, "logout must redirect");
    let cleared = headers
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        cleared.contains("Max-Age=0"),
        "logout must clear cookie: {cleared}"
    );

    let _ = user.delete().await;
}

#[tokio::test]
async fn token_rejects_wrong_pkce_verifier() {
    if !provider_available().await {
        return;
    }
    let user = TestUser::new("apiofpkc");
    let (status, _) = user.create().await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }
    let redirect_uri = portail_core::config::Config::global()
        .oidc_redirect_uri
        .as_str();

    let url = auth_url(
        portail_core::config::Config::global()
            .oidc_client_id
            .as_str(),
        redirect_uri,
        "stpkce",
    );
    let (_, headers, _, _) = send_raw(Method::GET, &url, &[], None).await;
    let location = headers
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let req_id = location
        .split("req=")
        .nth(1)
        .unwrap_or_default()
        .split('&')
        .next()
        .unwrap_or_default()
        .to_string();

    let (_, _, body, _) = send_raw(
        Method::POST,
        "/api/oidc/login",
        &header_pairs(&[("content-type", "application/json".to_string())]),
        Some(
            json!({
                "req": req_id,
                "username": user.uid,
                "password": user.password,
            })
            .to_string(),
        ),
    )
    .await;
    let redirect = body["redirect"].as_str().unwrap_or_default().to_string();
    let code = redirect
        .split("code=")
        .nth(1)
        .unwrap_or_default()
        .split('&')
        .next()
        .unwrap_or_default()
        .to_string();

    let Some((client_id, secret)) = oidc_client_secret() else {
        return;
    };
    let basic = base64::engine::general_purpose::STANDARD.encode(format!("{client_id}:{secret}"));
    let (status, _, body, _) = send_raw(
        Method::POST,
        "/api/oidc/token",
        &header_pairs(&[
            (
                "content-type",
                "application/x-www-form-urlencoded".to_string(),
            ),
            ("authorization", format!("Basic {basic}")),
        ]),
        Some(format!(
            "grant_type=authorization_code&code={}&redirect_uri={}&code_verifier=wrong-verifier",
            urlencoding(&code),
            urlencoding(redirect_uri),
        )),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "bad PKCE must be 400");
    assert_eq!(
        body["error"], "invalid_grant",
        "PKCE failure is invalid_grant"
    );

    let _ = user.delete().await;
}

#[tokio::test]
async fn login_rejects_wrong_password() {
    if !provider_available().await {
        return;
    }
    let user = TestUser::new("apiofwrp");
    let (status, _) = user.create().await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }
    let redirect_uri = portail_core::config::Config::global()
        .oidc_redirect_uri
        .as_str();

    let url = auth_url(
        portail_core::config::Config::global()
            .oidc_client_id
            .as_str(),
        redirect_uri,
        "stwrp",
    );
    let (_, headers, _, _) = send_raw(Method::GET, &url, &[], None).await;
    let location = headers
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let req_id = req_id_from(&location);

    let (status, _, body, _) = send_raw(
        Method::POST,
        "/api/oidc/login",
        &header_pairs(&[("content-type", "application/json".to_string())]),
        Some(
            json!({
                "req": req_id,
                "username": user.uid,
                "password": "wrong-password",
            })
            .to_string(),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "bad password must be 401");
    assert_eq!(
        body["error"], "invalid username or password",
        "JSON error contract expected: {body}"
    );

    let _ = user.delete().await;
}

#[tokio::test]
async fn login_same_req_still_works_after_wrong_password() {
    if !provider_available().await {
        return;
    }
    let user = TestUser::new("apiolsw");
    let (status, _) = user.create().await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }
    assert_eq!(status, StatusCode::CREATED);
    let redirect_uri = portail_core::config::Config::global()
        .oidc_redirect_uri
        .as_str();
    let url = auth_url(
        portail_core::config::Config::global()
            .oidc_client_id
            .as_str(),
        redirect_uri,
        "stlsw",
    );
    let (_, headers, _, _) = send_raw(Method::GET, &url, &[], None).await;
    let location = headers
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let req_id = req_id_from(&location);

    let (status, _, _, _) = send_raw(
        Method::POST,
        "/api/oidc/login",
        &header_pairs(&[("content-type", "application/json".to_string())]),
        Some(
            json!({
                "req": req_id,
                "username": user.uid,
                "password": "wrong-password",
            })
            .to_string(),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "bad password must be 401");

    let (status, _, body, _) = send_raw(
        Method::POST,
        "/api/oidc/login",
        &header_pairs(&[("content-type", "application/json".to_string())]),
        Some(
            json!({
                "req": req_id,
                "username": user.uid,
                "password": user.password,
            })
            .to_string(),
        ),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "retry with correct password on same req must succeed: {body}"
    );
    let redirect = body["redirect"].as_str().unwrap_or_default().to_string();
    assert!(
        redirect.starts_with(redirect_uri),
        "redirect must target the registered redirect_uri: {redirect}"
    );

    let _ = user.delete().await;
}

#[tokio::test]
async fn login_rejects_unknown_req() {
    let (status, _, body, _) = send_raw(
        Method::POST,
        "/api/oidc/login",
        &header_pairs(&[("content-type", "application/json".to_string())]),
        Some(
            json!({
                "req": "deadbeefdeadbeefdeadbeefdeadbeef",
                "username": "someone",
                "password": "pw",
            })
            .to_string(),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "unknown req must be 400");
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("expired"),
        "english JSON error expected: {body}"
    );
}

#[tokio::test]
async fn authorize_requires_registered_redirect_uri() {
    let url = auth_url(
        portail_core::config::Config::global()
            .oidc_client_id
            .as_str(),
        "https://evil.example/cb",
        "stevil",
    );
    let (status, _, _, _) = send_raw(Method::GET, &url, &[], None).await;

    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "unregistered redirect_uri must be 400"
    );
}

#[cfg(feature = "test-api")]
#[tokio::test]
async fn oidc_connection_test_reports_ok_for_configured_client() {
    let (status, _, body, _) = send_raw(
        Method::POST,
        "/api/auth/oidc-test",
        &header_pairs(&[("content-type", "application/json".to_string())]),
        Some(json!({}).to_string()),
    )
    .await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    assert_eq!(status, StatusCode::OK, "self-test should answer 200");
    assert!(
        body["client_found"].as_bool().unwrap_or(false),
        "the portal client must be found: {body}"
    );
    assert!(
        body["token_issued"].as_bool().unwrap_or(false),
        "exchange with stored credentials must succeed: {body}"
    );
    assert!(
        body["userinfo_ok"].as_bool().unwrap_or(false),
        "issued token must verify locally: {body}"
    );
}

#[cfg(feature = "test-api")]
#[tokio::test]
async fn oidc_connection_test_reports_unknown_client() {
    let (status, _, body, _) = send_raw(
        Method::POST,
        "/api/auth/oidc-test",
        &header_pairs(&[("content-type", "application/json".to_string())]),
        Some(json!({"client_id": "ghost-client"}).to_string()),
    )
    .await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    assert_eq!(status, StatusCode::OK, "report mode, not HTTP error");
    assert!(
        !body["client_found"].as_bool().unwrap_or(true),
        "unknown client must be reported not found: {body}"
    );
}
