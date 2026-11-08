use serde::{Serialize, Serializer};

const MEGABIT_PER_SECOND: u64 = 1_000_000;
const GIGABIT_PER_SECOND: u64 = MEGABIT_PER_SECOND * 1_000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BitsPerSecond(pub u64);

impl BitsPerSecond {
    pub const fn from_bits_per_second(value: u64) -> Self {
        Self(value)
    }

    pub const fn from_megabits_per_second(value: u64) -> Self {
        Self(value * MEGABIT_PER_SECOND)
    }

    pub const fn megabits(self) -> u64 {
        self.0 / MEGABIT_PER_SECOND
    }

    pub fn scaled(self) -> String {
        let (tier, unit) = if self.0 < MEGABIT_PER_SECOND {
            (1u64, "bit/s")
        } else if self.0 < GIGABIT_PER_SECOND {
            (MEGABIT_PER_SECOND, "Mib/s")
        } else {
            (GIGABIT_PER_SECOND, "Gib/s")
        };
        let whole = self.0 / tier;
        let tenth = (self.0 % tier).saturating_mul(10) / tier;
        if tenth == 0 {
            format!("{whole} {unit}")
        } else {
            format!("{whole}.{tenth} {unit}")
        }
    }
}

impl Serialize for BitsPerSecond {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.scaled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_per_second_converts_to_whole_megabits() {
        let giga = BitsPerSecond::from_megabits_per_second(1000);

        assert_eq!(giga.0, 1_000_000_000, "1000 Mbit/s is 1e9 bit/s");
        assert_eq!(giga.megabits(), 1000, "megabits round-trip");
    }

    #[test]
    fn bits_per_second_serializes_auto_scaled_with_unit() {
        let cases = [
            (BitsPerSecond(0), "0 bit/s"),
            (BitsPerSecond::from_megabits_per_second(100), "100 Mib/s"),
            (BitsPerSecond::from_megabits_per_second(1000), "1 Gib/s"),
            (
                BitsPerSecond::from_bits_per_second(2_500_000_000),
                "2.5 Gib/s",
            ),
        ];

        let serialized: Vec<String> = cases
            .iter()
            .map(|(rate, _)| {
                serde_json::to_value(rate)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect();

        assert_eq!(
            serialized,
            vec!["0 bit/s", "100 Mib/s", "1 Gib/s", "2.5 Gib/s"],
            "never 1000 Mib for 1 Gib"
        );
    }
}
