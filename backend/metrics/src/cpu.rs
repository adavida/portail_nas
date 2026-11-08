use serde::Serialize;

use crate::protocol::{CPU_IDLE_COLUMN, CPU_IOWAIT_COLUMN};
use portail_units::Percent;

#[derive(Serialize)]
pub struct Core {
    id: u32,
    usage: Percent,
}

fn cpu_totals(line: &str) -> Option<(u64, u64)> {
    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|value| value.parse().ok())
        .collect();
    let total = values.iter().sum();
    let idle = values
        .get(CPU_IDLE_COLUMN)?
        .saturating_add(*values.get(CPU_IOWAIT_COLUMN)?);
    Some((total, idle))
}

fn is_core_line(line: &str) -> bool {
    line.starts_with("cpu") && matches!(line.as_bytes().get(3), Some(byte) if byte.is_ascii_digit())
}

pub(crate) fn parse_cpu_totals(raw: &str) -> Option<(u64, u64)> {
    raw.lines()
        .find(|line| line.starts_with("cpu "))
        .and_then(cpu_totals)
}

pub(crate) fn parse_core_percents(before: &str, after: &str) -> Vec<Core> {
    before
        .lines()
        .filter(|line| is_core_line(line))
        .filter_map(|line| {
            let name = line.split_whitespace().next()?;
            let totals_before = cpu_totals(line)?;
            let line_after = after
                .lines()
                .find(|candidate| candidate.split_whitespace().next() == Some(name))?;
            let totals_after = cpu_totals(line_after)?;
            Some(Core {
                id: name[3..].parse().ok()?,
                usage: cpu_percent(totals_before, totals_after),
            })
        })
        .collect()
}

pub(crate) fn cpu_percent(before: (u64, u64), after: (u64, u64)) -> Percent {
    let total_delta = after.0.saturating_sub(before.0);
    let idle_delta = after.1.saturating_sub(before.1).min(total_delta);
    Percent::of(total_delta - idle_delta, total_delta)
}

pub(crate) fn count_cores(raw: &str) -> u32 {
    raw.lines().filter(|line| is_core_line(line)).count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cpu_totals_sums_all_and_idle_with_iowait() {
        let parsed = parse_cpu_totals("cpu  1 2 3 4 5 6 7 8\n");

        assert_eq!(
            parsed,
            Some((36, 9)),
            "total is the sum, idle includes iowait"
        );
    }

    #[test]
    fn cpu_percent_from_jiffies_delta() {
        let percent = cpu_percent((100, 50), (200, 100));

        assert_eq!(
            percent,
            Percent(50),
            "half of the delta idle means 50 percent busy"
        );
    }

    #[test]
    fn parse_core_percents_matches_cores_by_exact_name() {
        let before = "cpu0 0 0 0 100 0\ncpu1 0 0 0 100 0\n";
        let after = "cpu  0 0 0\ncpu0 100 0 0 100 0\ncpu1 0 0 0 200 0\n";

        let cores = parse_core_percents(before, after);

        assert_eq!(cores.len(), 2, "both cores should be reported");
        assert_eq!(cores[0].id, 0, "first core id");
        assert_eq!(cores[0].usage, Percent(100), "fully busy core");
        assert_eq!(cores[1].usage, Percent(0), "fully idle core");
    }

    #[test]
    fn count_cores_counts_cpu_numbered_lines() {
        let raw = "cpu  1 2 3\ncpu0 1 2 3\ncpu1 1 2 3\nintr 99\n";

        assert_eq!(count_cores(raw), 2, "cpu0 and cpu1 are cores");
    }
}
