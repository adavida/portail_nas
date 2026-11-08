use serde::{Serialize, Serializer};

use crate::per_second::per_second;
use crate::seconds::Seconds;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CountRate(pub u64);

impl CountRate {
    pub fn per_second(delta: u64, elapsed: Seconds) -> Self {
        Self(per_second(delta, elapsed))
    }
}

impl Serialize for CountRate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u64(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_rate_scales_count_deltas_over_windows() {
        assert_eq!(
            CountRate::per_second(4, Seconds::from_milliseconds(200)).0,
            20,
            "4 counts over 200 ms is 20 per second"
        );
    }

    #[test]
    fn count_rate_serializes_as_plain_integer() {
        let mut buffer = Vec::new();

        serde_json::to_writer(&mut buffer, &CountRate(1500)).unwrap();

        assert_eq!(String::from_utf8(buffer).unwrap(), "1500");
    }
}
