use std::fs;
use std::path::{Path, PathBuf};

use crate::read_sys_i64;

pub(crate) struct HwmonValue {
    pub(crate) name: String,
    pub(crate) raw: i64,
    pub(crate) crit_raw: Option<i64>,
}

fn hwmon_sensor_files() -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir("/sys/class/hwmon") else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let base = entry.path();
            let name = fs::read_to_string(base.join("name"))
                .map(|value| value.trim().to_string())
                .unwrap_or_else(|_| {
                    base.file_name()
                        .map(|value| value.to_string_lossy().to_string())
                        .unwrap_or_default()
                });
            let Ok(sensors) = fs::read_dir(&base) else {
                return None;
            };
            Some(
                sensors
                    .flatten()
                    .filter(|sensor| sensor.file_name().to_string_lossy().ends_with("_input"))
                    .map(|sensor| (name.clone(), sensor.path()))
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .collect()
}

fn sensor_label(base_dir: &Path, stem: &str, fallback: &str) -> String {
    fs::read_to_string(base_dir.join(format!("{stem}_label")))
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|_| fallback.to_string())
}

pub(crate) fn collect_hwmon(kind: &str) -> Vec<HwmonValue> {
    hwmon_sensor_files()
        .into_iter()
        .filter(|(_, path)| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with(kind))
        })
        .map(|(chip, path)| {
            let stem = path
                .file_name()
                .map(|name| name.to_string_lossy().replace("_input", ""))
                .unwrap_or_default();
            let base_dir = path.parent().unwrap_or(Path::new("/"));
            let label = sensor_label(base_dir, &stem, &chip);
            let name = if label == chip {
                chip
            } else {
                format!("{chip} {label}")
            };
            HwmonValue {
                name,
                raw: read_sys_i64(&path),
                crit_raw: fs::read_to_string(base_dir.join(format!("{stem}_crit")))
                    .ok()
                    .and_then(|value| value.trim().parse::<i64>().ok()),
            }
        })
        .collect()
}

pub(crate) fn unique_names(values: &[HwmonValue]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for value in values {
        let mut candidate = value.name.clone();
        let mut suffix = 2;
        while names.contains(&candidate) {
            candidate = format!("{} {}", value.name, suffix);
            suffix += 1;
        }
        names.push(candidate);
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(name: &str, raw: i64) -> HwmonValue {
        HwmonValue {
            name: name.to_string(),
            raw,
            crit_raw: None,
        }
    }

    #[test]
    fn unique_names_suffixes_duplicates() {
        let values = vec![
            value("nvme Composite", 40),
            value("nvme Composite", 42),
            value("k10temp Tctl", 55),
        ];

        let names = unique_names(&values);

        assert_eq!(
            names,
            vec![
                "nvme Composite".to_string(),
                "nvme Composite 2".to_string(),
                "k10temp Tctl".to_string()
            ],
            "duplicate sensor names get a numeric suffix"
        );
    }
}
