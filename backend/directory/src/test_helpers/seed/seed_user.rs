use crate::domain::users::Uid;

use super::{gen_uid::gen_uid, seed_and_create_user::seed_and_create_user};

pub(crate) async fn seed_user(prefix: &str) -> Option<Uid> {
    let uid = gen_uid(prefix);
    if seed_and_create_user(&uid, "Seed", "", "pw").await {
        Some(uid)
    } else {
        None
    }
}
