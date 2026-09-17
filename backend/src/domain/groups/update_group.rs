use serde::Deserialize;

use super::Description;
use crate::domain::shared::Name;

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateGroup {
    pub name: Name,
    pub description: Description,
}
