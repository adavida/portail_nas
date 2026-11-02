use crate::common::oidc::{challenge::challenge, urlencoding::urlencoding};

pub(crate) fn auth_url(client_id: &str, redirect_uri: &str, state: &str) -> String {
    format!(
        "/api/oidc/authorization?client_id={}&redirect_uri={}&response_type=code&state={state}&scope=openid%20groups%20email%20profile&code_challenge={}&code_challenge_method=S256",
        client_id,
        urlencoding(redirect_uri),
        urlencoding(&challenge()),
    )
}
