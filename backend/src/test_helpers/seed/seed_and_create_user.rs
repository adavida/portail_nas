use crate::{domain::users::Uid, error::AppError, repository::ldap::create_user};

use super::{ensure_clean::ensure_clean, new_user::new_user};

fn is_ldap_down<T>(r: &Result<T, AppError>) -> bool {
    matches!(r, Err(AppError::Ldap(_)))
}

pub(crate) async fn seed_and_create_user(
    uid: &Uid,
    name: &str,
    email: &str,
    password: &str,
) -> bool {
    ensure_clean(uid).await;

    let new = new_user(uid, name, email, password);

    let r = create_user(new).await;
    if is_ldap_down(&r) {
        return false;
    }

    r.unwrap();
    true
}
