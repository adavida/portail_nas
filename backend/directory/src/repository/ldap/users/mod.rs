mod authenticate_user;
mod create_user;
mod delete_user;
mod find_user_dn;
mod list_users;
mod update_user;
mod update_user_password;

pub use authenticate_user::authenticate_user;
pub use create_user::create_user;
pub use delete_user::delete_user;
pub use find_user_dn::find_user_dn;
pub use list_users::list_users;
pub use update_user::update_user;
pub use update_user_password::update_user_password;

use std::collections::HashSet;

pub(crate) fn one_set(v: String) -> HashSet<String> {
    [v].into_iter().collect()
}
