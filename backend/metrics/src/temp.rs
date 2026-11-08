use serde::Serialize;

use crate::hwmon::{collect_hwmon, unique_names};
use portail_units::Celsius;

#[derive(Serialize)]
pub struct Temp {
    name: String,
    celsius: Celsius,
    crit_celsius: Option<Celsius>,
}

pub(crate) fn read_temps() -> Vec<Temp> {
    let measured = collect_hwmon("temp");
    let mut temps: Vec<Temp> = unique_names(&measured)
        .into_iter()
        .zip(measured)
        .map(|(name, value)| Temp {
            name,
            celsius: Celsius::from_millidegrees(value.raw),
            crit_celsius: value.crit_raw.map(Celsius::from_millidegrees),
        })
        .collect();
    temps.sort_by(|a, b| a.name.cmp(&b.name));
    temps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialized_temp_reports_celsius_with_unit() {
        let reading = Temp {
            name: "k10temp Tctl".to_string(),
            celsius: Celsius::from_millidegrees(45_500),
            crit_celsius: Some(Celsius::from_millidegrees(90_000)),
        };

        let json = serde_json::to_value(&reading).unwrap();

        assert_eq!(
            json["celsius"], "45.5 °C",
            "45.5 °C serializes as decimal with unit"
        );
        assert_eq!(
            json["crit_celsius"], "90 °C",
            "the crit alarm is exposed for the gauge"
        );
    }

    #[test]
    fn temp_without_crit_alarm_serializes_null() {
        let reading = Temp {
            name: "nvme Composite".to_string(),
            celsius: Celsius::from_millidegrees(40_000),
            crit_celsius: None,
        };

        let json = serde_json::to_value(&reading).unwrap();

        assert!(
            json["crit_celsius"].is_null(),
            "a sensor without a crit alarm reports null"
        );
    }
}
