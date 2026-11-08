use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Percent(pub u8);

impl Percent {
    pub fn of(part: u64, whole: u64) -> Self {
        Percent((part.saturating_mul(100) / whole.max(1)).min(100) as u8)
    }

    pub fn scaled(self) -> String {
        format!("{} %", self.0)
    }
}

impl Serialize for Percent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.scaled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_of_truncates_and_clamps_to_one_hundred() {
        assert_eq!(Percent::of(50, 100), Percent(50), "half is 50 percent");
        assert_eq!(Percent::of(2, 3), Percent(66), "two thirds truncates");
        assert_eq!(
            Percent::of(150, 100),
            Percent(100),
            "over-count clamps to 100"
        );
        assert_eq!(Percent::of(5, 0), Percent(100), "zero whole cannot divide");
        assert_eq!(Percent::of(0, 100), Percent(0), "nothing used is 0 percent");
    }

    #[test]
    fn percent_serializes_with_percent_sign() {
        let json = serde_json::to_value(Percent(42)).unwrap();

        assert_eq!(json, serde_json::json!("42 %"), "unit carried in the value");
    }
}
