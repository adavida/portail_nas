use serde::{Serialize, Serializer};

const MILLIDEGREES_PER_DEGREE: i64 = 1_000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Celsius(i64);

impl Celsius {
    pub const fn from_millidegrees(value: i64) -> Self {
        Self(value)
    }

    pub const fn millidegrees(self) -> i64 {
        self.0
    }

    pub fn scaled(self) -> String {
        let whole = self.0 / MILLIDEGREES_PER_DEGREE;
        let tenth = (self.0.abs() % MILLIDEGREES_PER_DEGREE) * 10 / MILLIDEGREES_PER_DEGREE;
        let sign = if self.0 < 0 && whole == 0 { "-" } else { "" };
        if tenth == 0 {
            format!("{whole} °C")
        } else {
            format!("{sign}{whole}.{tenth} °C")
        }
    }
}

impl Serialize for Celsius {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.scaled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn celsius_keeps_millidegree_precision_including_negative_values() {
        let freezing = Celsius::from_millidegrees(-5_500);
        let warm = Celsius::from_millidegrees(45_500);

        assert_eq!(
            freezing.millidegrees(),
            -5_500,
            "sub-zero sensor stays exact"
        );
        assert_eq!(warm.millidegrees(), 45_500);
    }

    #[test]
    fn celsius_serializes_as_decimal_with_degree_unit() {
        let cases = [
            (Celsius::from_millidegrees(45_500), "45.5 °C"),
            (Celsius::from_millidegrees(90_000), "90 °C"),
            (Celsius::from_millidegrees(-5_500), "-5.5 °C"),
            (Celsius::from_millidegrees(-500), "-0.5 °C"),
        ];

        let serialized: Vec<String> = cases
            .iter()
            .map(|(value, _)| {
                serde_json::to_value(value)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect();

        assert_eq!(
            serialized,
            vec!["45.5 °C", "90 °C", "-5.5 °C", "-0.5 °C"],
            "human readable signed degrees"
        );
    }
}
