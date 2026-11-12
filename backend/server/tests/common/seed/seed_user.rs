use portail_core::error::AppError;
use portail_directory::domain::ldap::{Email, Name, Password, Uid};
use portail_directory::domain::users::NewUser;
use portail_directory::repository::{create_user, delete_user};

pub async fn seed_user(uid_str: &str) -> Result<(), AppError> {
    let uid = Uid::try_new(uid_str.to_string()).map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = delete_user(&uid).await;

    let new = NewUser {
        uid,
        name: Name::try_new("Seed Member".into()).map_err(|e| AppError::Internal(e.to_string()))?,
        email: Email::try_new("".into()).map_err(|e| AppError::Internal(e.to_string()))?,
        password: Password::try_new("secret123".into())
            .map_err(|e| AppError::Internal(e.to_string()))?,
    };

    create_user(new).await.map(|_| ())
}
