use std::fs;

use serde::Serialize;

use crate::read_sys_u64;
use portail_units::BitsPerSecond;

#[derive(Serialize)]
pub struct Interface {
    name: String,
    is_up: bool,
    speed: BitsPerSecond,
    rx_dropped: u64,
    rx_errors: u64,
}

pub(crate) fn read_interfaces() -> Vec<Interface> {
    let Ok(entries) = fs::read_dir("/sys/class/net") else {
        return Vec::new();
    };
    let mut interfaces: Vec<Interface> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "lo" {
                return None;
            }
            let base = entry.path();
            Some(Interface {
                is_up: fs::read_to_string(base.join("operstate"))
                    .unwrap_or_default()
                    .trim()
                    == "up",
                speed: BitsPerSecond::from_megabits_per_second(read_sys_u64(&base.join("speed"))),
                rx_dropped: read_sys_u64(&base.join("statistics/rx_dropped")),
                rx_errors: read_sys_u64(&base.join("statistics/rx_errors")),
                name,
            })
        })
        .collect();
    interfaces.sort_by(|a, b| a.name.cmp(&b.name));
    interfaces
}
