use axum::{Extension, Json};

use crate::auth::Claims;
use crate::auth::middleware::AuthUser;

#[derive(serde::Serialize)]
pub struct Me {
    sub: String,
    preferred_username: Option<String>,
    name: Option<String>,
    email: Option<String>,
    groups: Vec<String>,
    is_admin: bool,
}

impl Me {
    fn from_claims(claims: &Claims) -> Self {
        Self {
            sub: claims.sub.clone(),
            preferred_username: claims.preferred_username.clone(),
            name: None,
            email: claims.email.clone(),
            groups: claims.groups.clone(),
            is_admin: claims.is_admin(),
        }
    }
}

pub async fn me(Extension(AuthUser(claims)): Extension<AuthUser>) -> Json<Me> {
    let mut me = Me::from_claims(&claims);
    if let Some(username) = claims.preferred_username.clone() {
        let fresh = portail_directory::controllers::users::list()
            .await
            .ok()
            .and_then(|users| users.into_iter().find(|u| u.uid.as_str() == username));
        if let Some(user) = fresh {
            me.name = Some(user.name.as_str().to_string());
            me.email = Some(user.email.as_str().to_string());
        }
    }
    Json(me)
}
