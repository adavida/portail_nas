use crate::seconds::{MILLIS_PER_SECOND, Seconds};

pub fn per_second(delta: u64, elapsed: Seconds) -> u64 {
    delta.saturating_mul(MILLIS_PER_SECOND) / elapsed.0.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_second_scales_millisecond_windows() {
        assert_eq!(
            per_second(6000, Seconds::from_milliseconds(200)),
            30_000,
            "6000 units over 200 ms is 30000 per second"
        );
    }

    #[test]
    fn per_second_clamps_zero_elapsed_to_one_millisecond() {
        assert_eq!(
            per_second(500, Seconds::from_milliseconds(0)),
            500_000,
            "zero elapsed must not divide by zero"
        );
    }
}
