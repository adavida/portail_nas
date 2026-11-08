pub mod cpu;
pub mod disk;
pub mod disk_io;
pub mod fan;
pub mod hwmon;
pub mod interface;
pub mod memory;
pub mod net;
pub mod protocol;
pub mod raid;
pub mod system;
pub mod tcp;
pub mod temp;

pub use system::{System, snapshot};

use std::path::Path;

pub(crate) fn read_sys_u64(path: &Path) -> u64 {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or_default()
}

pub(crate) fn read_sys_i64(path: &Path) -> i64 {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or_default()
}

pub(crate) fn col_u64(cols: &[&str], index: usize) -> u64 {
    cols.get(index)
        .and_then(|value| value.parse().ok())
        .unwrap_or_default()
}
