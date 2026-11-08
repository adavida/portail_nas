use serde::Serialize;

use portail_units::Hundredths;

#[derive(Serialize)]
pub struct Raid {
    name: String,
    state: String,
    resync: Option<Hundredths>,
}

pub(crate) fn parse_mdstat(raw: &str) -> Vec<Raid> {
    let mut raids: Vec<Raid> = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("md") {
            let Some(colon) = trimmed.find(" : ") else {
                continue;
            };
            raids.push(Raid {
                name: trimmed[..colon].to_string(),
                state: trimmed[colon + 3..]
                    .split_whitespace()
                    .next()
                    .unwrap_or("unknown")
                    .to_string(),
                resync: None,
            });
        } else if trimmed.starts_with('[') {
            let Some(eq) = trimmed.find('=') else {
                continue;
            };
            let percent = trimmed[eq + 1..]
                .split_whitespace()
                .next()
                .and_then(|value| value.trim_end_matches('%').parse::<f32>().ok());
            if let (Some(raid), Some(percent)) = (raids.last_mut(), percent) {
                raid.resync = Some(Hundredths::from_f32(percent));
            }
        }
    }
    raids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mdstat_reports_state_and_resync_percent() {
        let raw = "Personalities : [raid1]\nmd0 : active raid1 sdb1[1] sda1[0]\n      9765312 blocks [2/2] [UU]\n      [>....] resync = 45.3% (4427776/9765312) finish=11.2min speed=7915K/sec\n\nmd1 : inactive sdc1[0](S)\n      1024 blocks\n\n";

        let raids = parse_mdstat(raw);

        assert_eq!(raids.len(), 2, "both md devices kept");
        assert_eq!(raids[0].name, "md0");
        assert_eq!(raids[0].state, "active");
        assert_eq!(raids[0].resync, Some(Hundredths(4530)), "resync parsed");
        assert_eq!(raids[1].state, "inactive");
        assert_eq!(raids[1].resync, None, "no progress line");
    }
}
