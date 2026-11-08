use serde::{Serialize, Serializer};

pub(crate) const MILLIS_PER_SECOND: u64 = 1_000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Seconds(pub u64);

impl Seconds {
    pub const fn from_milliseconds(value: u64) -> Self {
        Self(value)
    }

    pub const fn from_seconds(value: u64) -> Self {
        Self(value * MILLIS_PER_SECOND)
    }

    pub const fn seconds(self) -> u64 {
        self.0 / MILLIS_PER_SECOND
    }

    pub fn scaled(self) -> String {
        if self.0 < MILLIS_PER_SECOND {
            return format!("{} ms", self.0);
        }
        let tenth = (self.0 % MILLIS_PER_SECOND) * 10 / MILLIS_PER_SECOND;
        if tenth == 0 {
            format!("{} s", self.seconds())
        } else {
            format!("{}.{} s", self.seconds(), tenth)
        }
    }
}

impl Serialize for Seconds {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.scaled())
    }
}

impl std::ops::Sub for Seconds {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_stores_milliseconds_and_saturates_on_decrease() {
        let ms = Seconds::from_milliseconds(200);
        let secs = Seconds::from_seconds(3);

        assert_eq!(ms.0, 200, "200 ms is stored as 200 base milliseconds");
        assert_eq!(secs.seconds(), 3, "3 seconds round-trip");
        assert_eq!(
            (ms - Seconds::from_milliseconds(500)).0,
            0,
            "negative time delta saturates to zero"
        );
    }

    #[test]
    fn seconds_serializes_with_the_fitting_unit() {
        let cases = [
            (Seconds::from_milliseconds(250), "250 ms"),
            (Seconds::from_milliseconds(1_000), "1 s"),
            (Seconds::from_milliseconds(3_600_400), "3600.4 s"),
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
            vec!["250 ms", "1 s", "3600.4 s"],
            "never 1000 ms for 1 s"
        );
    }
}
