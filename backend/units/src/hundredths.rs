use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Hundredths(pub u32);

impl Hundredths {
    pub fn from_f32(value: f32) -> Self {
        Self((value * 100.0).round() as u32)
    }

    pub fn scaled(self) -> String {
        format!("{}.{:02}", self.0 / 100, self.0 % 100)
    }
}

impl Serialize for Hundredths {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.scaled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_f32_rounds_to_whole_hundredths() {
        assert_eq!(
            Hundredths::from_f32(2.345).0,
            235,
            "2.345 rounds to 235 hundredths"
        );
        assert_eq!(Hundredths::from_f32(0.5).0, 50, "exact decimal keeps value");
    }

    #[test]
    fn hundredths_serializes_as_two_decimal_string() {
        let json = serde_json::to_value(Hundredths(4530)).unwrap();
        let null_json = serde_json::to_string(&Option::<Hundredths>::None).unwrap();

        assert_eq!(json, serde_json::json!("45.30"), "fixed point exposed");
        assert_eq!(null_json, "null", "absent sensor stays JSON null");
    }
}
