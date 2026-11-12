pub mod ldap;
pub mod sqlite;

mod add_member;
mod authenticate_user;
mod create_group;
mod create_user;
mod delete_group;
mod delete_user;
mod find_user;
mod list_groups;
mod list_uids;
mod list_users;
mod remove_member;
mod update_group;
mod update_user;
mod update_user_password;

pub use add_member::add_member;
pub use authenticate_user::authenticate_user;
pub use create_group::create_group;
pub use create_user::create_user;
pub use delete_group::delete_group;
pub use delete_user::delete_user;
pub use find_user::find_user;
pub use list_groups::list_groups;
pub use list_uids::list_uids;
pub use list_users::list_users;
pub use remove_member::remove_member;
pub use update_group::update_group;
pub use update_user::update_user;
pub use update_user_password::update_user_password;

use crate::domain::ldap::{Gid, Uid};
use portail_core::error::AppError;

pub(crate) fn group_not_found(gid: &Gid) -> AppError {
    AppError::NotFound(format!("gid={} not found", gid.as_str()))
}

pub(crate) fn user_not_found(uid: &Uid) -> AppError {
    AppError::NotFound(format!("uid={} not found", uid.as_str()))
}
