use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Description(String);

impl Description {
    pub fn try_new(raw: String) -> Self {
        Self(raw.trim().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Description {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::try_new(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid() {
        let d = Description::try_new("Devs team".into());

        assert_eq!(d.as_str(), "Devs team");
    }

    #[test]
    fn trims_whitespace() {
        let d = Description::try_new("  Devs  ".into());

        assert_eq!(d.as_str(), "Devs", "description should be trimmed");
    }

    #[test]
    fn accepts_empty() {
        let d = Description::try_new("   ".into());

        assert_eq!(d.as_str(), "", "empty description is allowed for groups");
    }
}
