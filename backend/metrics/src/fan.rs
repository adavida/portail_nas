use serde::Serialize;

use crate::hwmon::{collect_hwmon, unique_names};
use portail_units::Rpm;

#[derive(Serialize)]
pub struct Fan {
    name: String,
    speed: Rpm,
}

pub(crate) fn read_fans() -> Vec<Fan> {
    let measured: Vec<_> = collect_hwmon("fan")
        .into_iter()
        .filter(|value| value.raw > 0)
        .collect();
    let mut fans: Vec<Fan> = unique_names(&measured)
        .into_iter()
        .zip(measured)
        .map(|(name, value)| Fan {
            name,
            speed: Rpm(value.raw as u64),
        })
        .collect();
    fans.sort_by(|a, b| a.name.cmp(&b.name));
    fans
}
