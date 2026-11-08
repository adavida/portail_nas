use serde::{Serialize, Serializer};

const KIBIBYTE: u64 = 1024;
pub(crate) const MEBIBYTE: u64 = KIBIBYTE * 1024;
const SECTOR_BYTES: u64 = 512;
const GIBIBYTE: u64 = MEBIBYTE * 1024;
const TEBIBYTE: u64 = GIBIBYTE * 1024;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bytes(pub u64);

impl Bytes {
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }

    pub const fn from_kibibytes(value: u64) -> Self {
        Self(value * KIBIBYTE)
    }

    pub const fn from_sectors(value: u64) -> Self {
        Self(value * SECTOR_BYTES)
    }

    pub const fn kibibytes(self) -> u64 {
        self.0 / KIBIBYTE
    }

    pub const fn mebibytes(self) -> u64 {
        self.0 / MEBIBYTE
    }

    pub fn scaled(self) -> String {
        scale_with_unit(self.0, false)
    }
}

impl Serialize for Bytes {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.scaled())
    }
}

impl std::fmt::Display for Bytes {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.scaled())
    }
}

pub(crate) fn scale_with_unit(value: u64, per_second: bool) -> String {
    let (tier, unit) = match value {
        v if v < KIBIBYTE => (1u64, "B"),
        v if v < MEBIBYTE => (KIBIBYTE, "KiB"),
        v if v < GIBIBYTE => (MEBIBYTE, "MiB"),
        v if v < TEBIBYTE => (GIBIBYTE, "GiB"),
        _ => (TEBIBYTE, "TiB"),
    };
    let unit_label = if per_second {
        format!("{unit}/s")
    } else {
        unit.to_string()
    };
    let whole = value / tier;
    let tenth = (value % tier).saturating_mul(10) / tier;
    if tenth == 0 {
        format!("{whole} {unit_label}")
    } else {
        format!("{whole}.{tenth} {unit_label}")
    }
}

impl std::ops::Sub for Bytes {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

impl std::ops::AddAssign for Bytes {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_converts_to_whole_units_and_saturates_on_decrease() {
        let kib = Bytes::from_kibibytes(2048);
        let sectors = Bytes::from_sectors(4);

        assert_eq!(kib.kibibytes(), 2048, "kibibytes round-trip");
        assert_eq!(kib.mebibytes(), 2, "2048 KiB is 2 MiB");
        assert_eq!(sectors.0, 2048, "4 sectors of 512 bytes");
        assert_eq!(
            (kib - Bytes::from_u64(4_000_000)).0,
            0,
            "counter reset must saturate to zero, not wrap"
        );
    }

    #[test]
    fn bytes_serializes_auto_scaled_with_unit() {
        let cases = [
            (Bytes::from_u64(900), "900 B"),
            (Bytes::from_kibibytes(2560), "2.5 MiB"),
            (Bytes::from_u64(3 * GIBIBYTE), "3 GiB"),
            (Bytes::from_u64(5 * TEBIBYTE), "5 TiB"),
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
            vec!["900 B", "2.5 MiB", "3 GiB", "5 TiB"],
            "largest fitting unit with one decimal"
        );
    }
}
