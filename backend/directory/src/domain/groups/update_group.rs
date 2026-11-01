use serde::Deserialize;

use crate::domain::ldap::{Description, Name};

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateGroup {
    pub name: Name,
    pub description: Description,
}
