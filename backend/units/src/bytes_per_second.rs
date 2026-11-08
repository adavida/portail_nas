use serde::{Serialize, Serializer};

use crate::bytes::{Bytes, MEBIBYTE, scale_with_unit};
use crate::per_second::per_second;
use crate::seconds::Seconds;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BytesPerSecond(pub u64);

impl BytesPerSecond {
    pub fn per_second(delta: Bytes, elapsed: Seconds) -> Self {
        Self(per_second(delta.0, elapsed))
    }

    pub const fn mebibytes_per_second(self) -> u64 {
        self.0 / MEBIBYTE
    }
}

impl Serialize for BytesPerSecond {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&scale_with_unit(self.0, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_per_second_scales_byte_deltas_over_windows() {
        let rate =
            BytesPerSecond::per_second(Bytes::from_u64(6000), Seconds::from_milliseconds(200));

        assert_eq!(rate.0, 30_000, "6000 bytes over 200 ms is 30000 B/s");
        assert_eq!(
            BytesPerSecond::per_second(
                Bytes::from_u64(4_000_000_000),
                Seconds::from_milliseconds(200)
            )
            .0,
            20_000_000_000,
            "gigabyte-scale deltas must not overflow"
        );
    }

    #[test]
    fn bytes_per_second_serializes_auto_scaled_with_unit() {
        let cases = [
            (BytesPerSecond(512), "512 B/s"),
            (BytesPerSecond(10_485_760), "10 MiB/s"),
            (
                BytesPerSecond::per_second(Bytes::from_u64(3 * MEBIBYTE), Seconds::from_seconds(1)),
                "3 MiB/s",
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
            vec!["512 B/s", "10 MiB/s", "3 MiB/s"],
            "largest fitting byte-rate unit"
        );
    }
}
