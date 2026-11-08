use std::time::Duration;

pub(crate) const CPU_SAMPLE_INTERVAL: Duration = Duration::from_millis(200);
pub(crate) const CPU_IDLE_COLUMN: usize = 3;
pub(crate) const CPU_IOWAIT_COLUMN: usize = 4;
pub(crate) const NET_RX_BYTES_COLUMN: usize = 0;
pub(crate) const NET_RX_ERRORS_COLUMN: usize = 2;
pub(crate) const NET_RX_DROPPED_COLUMN: usize = 3;
pub(crate) const NET_TX_BYTES_COLUMN: usize = 8;
pub(crate) const DISK_NAME_COLUMN: usize = 2;
pub(crate) const DISK_SECTORS_READ_COLUMN: usize = 5;
pub(crate) const DISK_SECTORS_WRITTEN_COLUMN: usize = 9;
pub(crate) const DISK_IO_MS_COLUMN: usize = 12;
pub(crate) const TCP_CURR_ESTAB_COLUMN: usize = 9;
pub(crate) const TCP_RETRANS_COLUMN: usize = 12;
