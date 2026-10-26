pub(crate) fn session_cookie(set_cookie: Option<&axum::http::HeaderValue>) -> String {
    set_cookie
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .split(';')
        .next()
        .unwrap_or_default()
        .to_string()
}
