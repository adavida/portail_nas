use crate::col_u64;
use crate::protocol::{
    NET_RX_BYTES_COLUMN, NET_RX_DROPPED_COLUMN, NET_RX_ERRORS_COLUMN, NET_TX_BYTES_COLUMN,
};
use portail_units::{Bytes, BytesPerSecond, Seconds};

#[derive(Default)]
pub(crate) struct NetTotals {
    rx_bytes: Bytes,
    rx_errors: u64,
    rx_dropped: u64,
    tx_bytes: Bytes,
}

pub(crate) struct NetRates {
    pub(crate) rx_bytes_per_sec: BytesPerSecond,
    pub(crate) tx_bytes_per_sec: BytesPerSecond,
    pub(crate) rx_dropped_delta: u64,
    pub(crate) rx_errors_delta: u64,
}

pub(crate) fn parse_net_totals(raw: &str) -> NetTotals {
    let mut totals = NetTotals::default();
    for line in raw.lines().skip(2) {
        let Some((iface, rest)) = line.split_once(':') else {
            continue;
        };
        if iface.trim() == "lo" {
            continue;
        }
        let cols: Vec<&str> = rest.split_whitespace().collect();
        totals.rx_bytes += Bytes::from_u64(col_u64(&cols, NET_RX_BYTES_COLUMN));
        totals.rx_errors += col_u64(&cols, NET_RX_ERRORS_COLUMN);
        totals.rx_dropped += col_u64(&cols, NET_RX_DROPPED_COLUMN);
        totals.tx_bytes += Bytes::from_u64(col_u64(&cols, NET_TX_BYTES_COLUMN));
    }
    totals
}

pub(crate) fn net_rates(before: NetTotals, after: NetTotals, elapsed: Seconds) -> NetRates {
    NetRates {
        rx_bytes_per_sec: BytesPerSecond::per_second(after.rx_bytes - before.rx_bytes, elapsed),
        tx_bytes_per_sec: BytesPerSecond::per_second(after.tx_bytes - before.tx_bytes, elapsed),
        rx_dropped_delta: after.rx_dropped.saturating_sub(before.rx_dropped),
        rx_errors_delta: after.rx_errors.saturating_sub(before.rx_errors),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_net_totals_skips_loopback_and_sums_interfaces() {
        let raw = "Inter-|   Receive                                                |  Transmit\n face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed\n    lo: 1000      10    0    0    0     0          0         0     1000      10    0    0    0     0       0          0\n  eth0: 2000      20    1    2    0     0          0         0     3000      30    0    0    0     0       0          0\n";

        let totals = parse_net_totals(raw);

        assert_eq!(totals.rx_bytes.0, 2000, "loopback should be excluded");
        assert_eq!(totals.tx_bytes.0, 3000, "tx bytes summed");
        assert_eq!(totals.rx_errors, 1, "rx errors summed");
        assert_eq!(totals.rx_dropped, 2, "rx dropped summed");
    }

    #[test]
    fn net_rates_scales_deltas_to_per_second() {
        let before = NetTotals {
            rx_bytes: Bytes::from_u64(1000),
            tx_bytes: Bytes::from_u64(500),
            ..Default::default()
        };
        let after = NetTotals {
            rx_bytes: Bytes::from_u64(7000),
            tx_bytes: Bytes::from_u64(1500),
            ..Default::default()
        };

        let rates = net_rates(before, after, Seconds::from_milliseconds(200));

        assert_eq!(rates.rx_bytes_per_sec.0, 30_000, "rx delta scaled");
        assert_eq!(rates.tx_bytes_per_sec.0, 5000, "tx delta scaled");
    }

    #[test]
    fn net_rates_saturates_on_counter_reset() {
        let before = NetTotals {
            rx_bytes: Bytes::from_u64(9000),
            ..Default::default()
        };
        let after = NetTotals {
            rx_bytes: Bytes::from_u64(1000),
            ..Default::default()
        };

        let rates = net_rates(before, after, Seconds::from_milliseconds(100));

        assert_eq!(
            rates.rx_bytes_per_sec.0, 0,
            "counter going down must report zero, not wrap"
        );
    }
}
