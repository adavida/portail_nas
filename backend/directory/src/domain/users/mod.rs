pub mod attr;
pub mod new_user;
pub mod update_user;
pub mod user;
pub mod user_error;

pub use attr::{UserAttr, errors::AttrError};
pub use new_user::NewUser;
pub use update_user::UpdateUser;
pub use user::User;
pub use user_error::UserError;
