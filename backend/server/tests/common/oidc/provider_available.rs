use axum::http::{Method, StatusCode};

use crate::common::http::send_raw::send_raw;

pub(crate) async fn provider_available() -> bool {
    let (status, _, _, _) = send_raw(Method::GET, "/jwks.json", &[], None).await;
    status == StatusCode::OK
}
