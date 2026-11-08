use serde::Serialize;

use portail_core::error::AppError;
use portail_units::Bytes;

#[derive(Serialize)]
pub struct Disk {
    mount: String,
    total: Bytes,
    used: Bytes,
}

// shortcut: df blocks the runtime a few ms, move to spawn_blocking under real load
pub(crate) fn run_df() -> Result<String, AppError> {
    let output = std::process::Command::new("df")
        .arg("-kP")
        .output()
        .map_err(|err| AppError::Internal(format!("run df failed: {err}")))?;
    if !output.status.success() {
        return Err(AppError::Internal("df exited with error".into()));
    }
    String::from_utf8(output.stdout)
        .map_err(|err| AppError::Internal(format!("df output not utf8: {err}")))
}

pub(crate) fn parse_df(raw: &str) -> Vec<Disk> {
    let mut disks: Vec<Disk> = Vec::new();
    for line in raw.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        let [source, total_kb, used_kb, _avail_kb, _capacity, mount @ ..] = cols.as_slice() else {
            continue;
        };
        if !source.starts_with("/dev/") {
            continue;
        }
        let (Ok(total), Ok(used)) = (total_kb.parse::<u64>(), used_kb.parse::<u64>()) else {
            continue;
        };
        let mount_path = mount.join(" ");
        if disks.iter().any(|disk| disk.mount == mount_path) {
            continue;
        }
        disks.push(Disk {
            mount: mount_path,
            total: Bytes::from_kibibytes(total),
            used: Bytes::from_kibibytes(used),
        });
    }
    disks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_df_keeps_block_devices_and_skips_pseudo_and_duplicates() {
        let raw = "Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/sda2 500000000 300000000 200000000 60% /\ntmpfs 1000 1 999 1% /dev/shm\n/dev/sdb1 1000000000 500000000 500000000 50% /data\ndevtmpfs 500 1 499 1% /dev\n";

        let disks = parse_df(raw);

        assert_eq!(disks.len(), 2, "only /dev/ sources kept");
        assert_eq!(disks[0].mount, "/");
        assert_eq!(disks[0].total.mebibytes(), 488_281);
        assert_eq!(disks[0].used.mebibytes(), 292_968);
        assert_eq!(disks[1].mount, "/data");
    }

    #[test]
    fn parse_df_dedupes_same_mount() {
        let raw = "Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/sda2 500000 300000 200000 60% /\n/dev/sda3 500000 100000 400000 20% /\n";

        let disks = parse_df(raw);

        assert_eq!(disks.len(), 1, "first mount wins on duplicate paths");
    }
}
