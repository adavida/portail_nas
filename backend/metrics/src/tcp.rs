use serde::Serialize;

use crate::col_u64;
use crate::protocol::{TCP_CURR_ESTAB_COLUMN, TCP_RETRANS_COLUMN};
use portail_units::{CountRate, Seconds};

#[derive(Serialize)]
pub struct TcpStats {
    established: u64,
    retrans_segments: CountRate,
}

fn parse_tcp_totals(raw: &str) -> (u64, u64) {
    let cols: Vec<&str> = raw
        .lines()
        .rfind(|line| line.starts_with("Tcp: "))
        .unwrap_or_default()
        .split_whitespace()
        .collect();
    (
        col_u64(&cols, TCP_CURR_ESTAB_COLUMN),
        col_u64(&cols, TCP_RETRANS_COLUMN),
    )
}

pub(crate) fn tcp_snapshot(before: &str, after: &str, elapsed: Seconds) -> TcpStats {
    let (_, before_retrans) = parse_tcp_totals(before);
    let (after_estab, after_retrans) = parse_tcp_totals(after);
    TcpStats {
        established: after_estab,
        retrans_segments: CountRate::per_second(
            after_retrans.saturating_sub(before_retrans),
            elapsed,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tcp_snapshot_reads_estab_and_retrans_rate() {
        let header = "Tcp: RtoAlgorithm RtoMin RtoMax MaxConn ActiveOpens PassiveOpens AttemptFails EstabResets CurrEstab InSegs OutSegs RetransSegs InErrs\n";
        let values = "Tcp: 2 1 200 0 10 5 2 1 7 100 200 4 0\n";

        let stats = tcp_snapshot(
            header,
            &format!("{header}{values}"),
            Seconds::from_milliseconds(200),
        );

        assert_eq!(stats.established, 7, "CurrEstab field");
        assert_eq!(
            stats.retrans_segments,
            CountRate(20),
            "4 retrans over 200 ms"
        );
    }
}
