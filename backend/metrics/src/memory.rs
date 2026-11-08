use portail_units::{Bytes, Hundredths, Seconds};

fn meminfo_value(raw: &str, name: &str) -> Option<Bytes> {
    raw.lines()
        .find(|line| line.starts_with(name))?
        .split_whitespace()
        .nth(1)?
        .parse::<u64>()
        .ok()
        .map(Bytes::from_kibibytes)
}

pub(crate) fn meminfo_bytes(raw: &str, name: &str) -> Bytes {
    meminfo_value(raw, name).unwrap_or_default()
}

pub(crate) fn parse_meminfo(raw: &str) -> Option<(Bytes, Bytes)> {
    let total = meminfo_value(raw, "MemTotal:")?;
    let available = meminfo_value(raw, "MemAvailable:").unwrap_or(total);
    Some((total, total - available))
}

pub(crate) fn parse_swap(raw: &str) -> (Bytes, Bytes) {
    let total = meminfo_value(raw, "SwapTotal:").unwrap_or_default();
    let free = meminfo_value(raw, "SwapFree:").unwrap_or(total);
    (total, total - free)
}

pub(crate) fn parse_loadavg(raw: &str) -> Option<(Hundredths, Hundredths, Hundredths)> {
    let mut fields = raw.split_whitespace();
    Some((
        Hundredths::from_f32(fields.next()?.parse().ok()?),
        Hundredths::from_f32(fields.next()?.parse().ok()?),
        Hundredths::from_f32(fields.next()?.parse().ok()?),
    ))
}

pub(crate) fn parse_uptime(raw: &str) -> Option<Seconds> {
    raw.split_whitespace()
        .next()?
        .split('.')
        .next()?
        .parse()
        .ok()
        .map(Seconds::from_seconds)
}

pub(crate) fn parse_pressure(raw: &str) -> Option<Hundredths> {
    raw.lines()
        .find(|line| line.starts_with("some"))?
        .split_whitespace()
        .find(|token| token.starts_with("avg10="))?
        .trim_start_matches("avg10=")
        .parse::<f32>()
        .ok()
        .map(Hundredths::from_f32)
}

pub(crate) fn parse_pgmajfault(raw: &str) -> u64 {
    raw.lines()
        .find(|line| line.starts_with("pgmajfault "))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse().ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_meminfo_computes_used_from_available() {
        let raw = "MemTotal:       16384 kB\nMemFree:         1024 kB\nMemAvailable:    4096 kB\n";

        let parsed = parse_meminfo(raw).map(|(total, used)| (total.mebibytes(), used.mebibytes()));

        assert_eq!(parsed, Some((16, 12)), "total and used should be in MB");
    }

    #[test]
    fn parse_meminfo_without_memtotal_returns_none() {
        let parsed = parse_meminfo("HugePages_Total:       0\n");

        assert!(parsed.is_none(), "missing MemTotal should fail parse");
    }

    #[test]
    fn meminfo_bytes_reads_named_field_as_bytes() {
        let raw = "Cached:          2048 kB\nBuffers:          512 kB\n";

        let cached = meminfo_bytes(raw, "Cached:");
        let buffers = meminfo_bytes(raw, "Buffers:");

        assert_eq!(
            cached.mebibytes(),
            2,
            "Cached should be parsed to mebibytes"
        );
        assert_eq!(
            buffers.0,
            512 * 1024,
            "sub-MB values keep their exact byte count"
        );
    }

    #[test]
    fn parse_swap_computes_used_from_free() {
        let raw = "SwapTotal:       4096 kB\nSwapFree:        1024 kB\n";

        let (total, used) = parse_swap(raw);

        assert_eq!(total.mebibytes(), 4, "swap total in MB");
        assert_eq!(used.mebibytes(), 3, "swap used in MB");
    }

    #[test]
    fn parse_swap_without_keys_returns_zeroes() {
        let (total, used) = parse_swap("MemTotal: 1 kB\n");

        assert_eq!(total.0, 0, "missing swap keys default to zero");
        assert_eq!(used.0, 0, "missing swap keys default to zero");
    }

    #[test]
    fn parse_loadavg_reads_first_three_fields() {
        let parsed = parse_loadavg("0.50 0.40 0.30 1/200 1234\n");

        assert_eq!(
            parsed,
            Some((Hundredths(50), Hundredths(40), Hundredths(30))),
            "load averages as fixed-point hundredths"
        );
    }

    #[test]
    fn parse_uptime_reads_whole_seconds() {
        let parsed = parse_uptime("3600.12 7200.00\n");

        assert_eq!(
            parsed,
            Some(Seconds::from_seconds(3600)),
            "fractional seconds are truncated"
        );
    }

    #[test]
    fn parse_pressure_reads_some_avg10() {
        let raw = "some avg10=2.50 avg60=1.00 avg300=0.50 total=123456\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n";

        let parsed = parse_pressure(raw);

        assert_eq!(parsed, Some(Hundredths(250)), "avg10 of the some line");
    }

    #[test]
    fn parse_pgmajfault_reads_counter() {
        let raw = "nr_dirty 10\npgmajfault 1234\npgpgin 5\n";

        let parsed = parse_pgmajfault(raw);

        assert_eq!(parsed, 1234, "pgmajfault counter value");
    }
}
