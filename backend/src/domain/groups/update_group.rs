use serde::Deserialize;

use super::{Description, Name};

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateGroup {
    pub name: Name,
    pub description: Description,
}
