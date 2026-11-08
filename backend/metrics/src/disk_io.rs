use serde::Serialize;

use crate::col_u64;
use crate::protocol::{
    DISK_IO_MS_COLUMN, DISK_NAME_COLUMN, DISK_SECTORS_READ_COLUMN, DISK_SECTORS_WRITTEN_COLUMN,
};
use portail_units::{Bytes, BytesPerSecond, Percent, Seconds};

#[derive(Serialize)]
pub struct DiskIo {
    name: String,
    read: BytesPerSecond,
    write: BytesPerSecond,
    busy: Percent,
}

#[derive(Clone, Copy)]
pub(crate) struct DiskTotals {
    sectors_read: Bytes,
    sectors_written: Bytes,
    io: Seconds,
}

fn is_physical_disk(name: &str) -> bool {
    if let Some(rest) = name.strip_prefix("nvme") {
        return !rest.contains('p');
    }
    let rest = match name.strip_prefix("sd").or_else(|| name.strip_prefix("vd")) {
        Some(rest) => rest,
        None => return false,
    };
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_lowercase())
}

pub(crate) fn parse_disk_stats(raw: &str) -> Vec<(String, DiskTotals)> {
    raw.lines()
        .filter_map(|line| {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() <= DISK_IO_MS_COLUMN {
                return None;
            }
            let name = cols[DISK_NAME_COLUMN];
            if !is_physical_disk(name) {
                return None;
            }
            Some((
                name.to_string(),
                DiskTotals {
                    sectors_read: Bytes::from_sectors(col_u64(&cols, DISK_SECTORS_READ_COLUMN)),
                    sectors_written: Bytes::from_sectors(col_u64(
                        &cols,
                        DISK_SECTORS_WRITTEN_COLUMN,
                    )),
                    io: Seconds::from_milliseconds(col_u64(&cols, DISK_IO_MS_COLUMN)),
                },
            ))
        })
        .collect()
}

pub(crate) fn parse_disk_io(
    before: &[(String, DiskTotals)],
    after: &[(String, DiskTotals)],
    elapsed: Seconds,
) -> Vec<DiskIo> {
    after
        .iter()
        .filter_map(|(name, totals)| {
            let previous = before
                .iter()
                .find(|(before_name, _)| before_name == name)
                .map(|(_, totals)| *totals)?;
            Some(DiskIo {
                name: name.clone(),
                read: BytesPerSecond::per_second(
                    totals.sectors_read - previous.sectors_read,
                    elapsed,
                ),
                write: BytesPerSecond::per_second(
                    totals.sectors_written - previous.sectors_written,
                    elapsed,
                ),
                busy: Percent::of((totals.io - previous.io).0, elapsed.0),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_physical_disk_keeps_disks_and_drops_partitions() {
        assert!(is_physical_disk("sda"), "sata disk");
        assert!(!is_physical_disk("sda1"), "sata partition");
        assert!(is_physical_disk("vdb"), "virtio disk");
        assert!(is_physical_disk("nvme0n1"), "nvme namespace");
        assert!(!is_physical_disk("nvme0n1p1"), "nvme partition");
        assert!(!is_physical_disk("loop0"), "loop device");
    }

    #[test]
    fn parse_disk_io_computes_rates_and_busy() {
        let before = vec![(
            "sda".to_string(),
            DiskTotals {
                sectors_read: Bytes::from_u64(0),
                sectors_written: Bytes::from_u64(0),
                io: Seconds::default(),
            },
        )];
        let after = vec![(
            "sda".to_string(),
            DiskTotals {
                sectors_read: Bytes::from_sectors(4096),
                sectors_written: Bytes::from_sectors(2048),
                io: Seconds::from_milliseconds(100),
            },
        )];

        let io = parse_disk_io(&before, &after, Seconds::from_milliseconds(200));

        assert_eq!(io.len(), 1, "one disk reported");
        assert_eq!(io[0].read.mebibytes_per_second(), 10, "2 MiB over 200 ms");
        assert_eq!(io[0].write.mebibytes_per_second(), 5, "1 MiB over 200 ms");
        assert_eq!(io[0].busy, Percent(50), "100 io-ms in a 200 ms window");
    }
}
