use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rpm(pub u64);

impl Rpm {
    pub fn scaled(self) -> String {
        format!("{} rpm", self.0)
    }
}

impl Serialize for Rpm {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.scaled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpm_serializes_with_unit_suffix() {
        let json = serde_json::to_value(Rpm(1200)).unwrap();

        assert_eq!(
            json,
            serde_json::json!("1200 rpm"),
            "sensor value with unit"
        );
    }
}
