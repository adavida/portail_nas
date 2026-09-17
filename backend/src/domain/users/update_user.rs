use serde::Deserialize;

use super::Email;
use crate::domain::shared::Name;

#[derive(Deserialize, Clone, Debug)]
pub struct UpdateUser {
    pub name: Name,
    pub email: Email,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_validates() {
        let json = r#"{"name":"","email":"a@ex.com"}"#;

        let err = serde_json::from_str::<UpdateUser>(json).unwrap_err();

        assert!(
            err.to_string().contains("name is empty"),
            "empty name should be rejected via Deserialize"
        );
    }
}
