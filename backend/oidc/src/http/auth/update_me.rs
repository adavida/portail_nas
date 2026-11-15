use axum::{Extension, Json};
use portail_core::error::AppError;
use portail_directory::{
    controllers::users::{update, update_attrs},
    domain::{
        ldap::{Email, Name},
        users::{UpdateUser, UserAttr},
    },
};
use serde::{Deserialize, Serialize};

use super::current_uid::current_uid;
use crate::auth::middleware::AuthUser;

#[derive(Deserialize)]
pub struct UpdateMe {
    name: String,
    email: String,
    #[serde(default)]
    attrs: Option<Vec<UserAttr>>,
}

#[derive(Serialize)]
pub struct MeSaveResult {
    profile_saved: bool,
    attrs_saved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn validated_profile(name: String, email: String) -> Result<UpdateUser, String> {
    Ok(UpdateUser {
        name: Name::try_new(name).map_err(|e| e.to_string())?,
        email: Email::try_new(email).map_err(|e| e.to_string())?,
    })
}

pub async fn update_me(
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(payload): Json<UpdateMe>,
) -> Result<Json<MeSaveResult>, AppError> {
    let uid = current_uid(&claims)?;
    let mut errors: Vec<String> = Vec::new();

    let profile_saved = match validated_profile(payload.name.clone(), payload.email.clone()) {
        Ok(data) => match update(uid.clone(), data).await {
            Ok(()) => true,
            Err(e) => {
                errors.push(e.to_string());
                false
            }
        },
        Err(e) => {
            errors.push(e);
            false
        }
    };

    let attrs_saved = match payload.attrs {
        None => true,
        Some(attrs) => match update_attrs(&uid, attrs).await {
            Ok(()) => true,
            Err(e) => {
                errors.push(e.to_string());
                false
            }
        },
    };

    Ok(Json(MeSaveResult {
        profile_saved,
        attrs_saved,
        error: if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attrs_default_to_none_and_shape_parses() {
        let payload: UpdateMe = serde_json::from_str(r#"{"name":"A","email":"a@ex.com"}"#).unwrap();

        assert!(payload.attrs.is_none(), "absent attrs means untouched");

        let with_attrs: UpdateMe = serde_json::from_str(
            r#"{"name":"A","email":"a@ex.com","attrs":[{"label":"city","value":"Nice"}]}"#,
        )
        .unwrap();

        assert_eq!(
            with_attrs.attrs.map(|entries| entries.len()),
            Some(1),
            "attrs should parse as a list of label/value pairs"
        );
    }

    #[test]
    fn validated_profile_rejects_empty_name_with_vo_message() {
        let err = validated_profile("".into(), "a@ex.com".into()).unwrap_err();

        assert!(
            err.contains("name is empty"),
            "the Name VO message should surface, got {err}"
        );
    }
}
