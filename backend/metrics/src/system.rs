use serde::Serialize;

use portail_core::error::AppError;

use crate::protocol::CPU_SAMPLE_INTERVAL;
use crate::{cpu, disk, disk_io, fan, interface, memory, net, raid, tcp, temp};
use portail_units::{Bytes, BytesPerSecond, CountRate, Hundredths, Percent, Seconds};

#[derive(Serialize)]
pub struct System {
    ram_total: Bytes,
    ram_used: Bytes,
    ram_cached: Bytes,
    ram_buffers: Bytes,
    ram_dirty: Bytes,
    swap_total: Bytes,
    swap_used: Bytes,
    swap_cached: Bytes,
    cpu_usage: Percent,
    cpu_cores: u32,
    cores: Vec<cpu::Core>,
    load_1: Hundredths,
    load_5: Hundredths,
    load_15: Hundredths,
    psi_cpu: Option<Hundredths>,
    psi_memory: Option<Hundredths>,
    psi_io: Option<Hundredths>,
    pgmajfault: CountRate,
    net_rx: BytesPerSecond,
    net_tx: BytesPerSecond,
    net_rx_dropped_delta: u64,
    net_rx_errors_delta: u64,
    interfaces: Vec<interface::Interface>,
    tcp: tcp::TcpStats,
    temps: Vec<temp::Temp>,
    fans: Vec<fan::Fan>,
    disks: Vec<disk::Disk>,
    disk_io: Vec<disk_io::DiskIo>,
    raid: Vec<raid::Raid>,
    uptime: Seconds,
    sensors: Vec<String>,
}

fn read_proc(path: &str) -> Result<String, AppError> {
    std::fs::read_to_string(path)
        .map_err(|err| AppError::Internal(format!("read {path} failed: {err}")))
}

pub async fn snapshot() -> Result<System, AppError> {
    let meminfo = read_proc("/proc/meminfo")?;
    let stat_before = read_proc("/proc/stat")?;
    let net_before = read_proc("/proc/net/dev").unwrap_or_default();
    let disk_before = read_proc("/proc/diskstats").unwrap_or_default();
    let vmstat_before = read_proc("/proc/vmstat").unwrap_or_default();
    let snmp_before = read_proc("/proc/net/snmp").unwrap_or_default();
    tokio::time::sleep(CPU_SAMPLE_INTERVAL).await;
    let elapsed = Seconds::from_milliseconds(CPU_SAMPLE_INTERVAL.as_millis() as u64);
    let stat_after = read_proc("/proc/stat")?;
    let net_after = read_proc("/proc/net/dev").unwrap_or_default();
    let disk_after = read_proc("/proc/diskstats").unwrap_or_default();
    let vmstat_after = read_proc("/proc/vmstat").unwrap_or_default();
    let snmp_after = read_proc("/proc/net/snmp").unwrap_or_default();
    let loadavg = read_proc("/proc/loadavg")?;
    let uptime = read_proc("/proc/uptime")?;
    let mdstat = read_proc("/proc/mdstat").unwrap_or_default();
    let df_output = disk::run_df()?;

    let (ram_total, ram_used) = memory::parse_meminfo(&meminfo)
        .ok_or_else(|| AppError::Internal("meminfo parse failed".into()))?;
    let (swap_total, swap_used) = memory::parse_swap(&meminfo);
    let cpu_before = cpu::parse_cpu_totals(&stat_before)
        .ok_or_else(|| AppError::Internal("stat parse failed".into()))?;
    let cpu_after = cpu::parse_cpu_totals(&stat_after)
        .ok_or_else(|| AppError::Internal("stat parse failed".into()))?;
    let net = net::net_rates(
        net::parse_net_totals(&net_before),
        net::parse_net_totals(&net_after),
        elapsed,
    );
    let (load_1, load_5, load_15) = memory::parse_loadavg(&loadavg)
        .ok_or_else(|| AppError::Internal("loadavg parse failed".into()))?;

    Ok(System {
        ram_total,
        ram_used,
        ram_cached: memory::meminfo_bytes(&meminfo, "Cached:"),
        ram_buffers: memory::meminfo_bytes(&meminfo, "Buffers:"),
        ram_dirty: memory::meminfo_bytes(&meminfo, "Dirty:"),
        swap_total,
        swap_used,
        swap_cached: memory::meminfo_bytes(&meminfo, "SwapCached:"),
        cpu_usage: cpu::cpu_percent(cpu_before, cpu_after),
        cpu_cores: cpu::count_cores(&stat_before),
        cores: cpu::parse_core_percents(&stat_before, &stat_after),
        load_1,
        load_5,
        load_15,
        psi_cpu: read_proc("/proc/pressure/cpu")
            .ok()
            .and_then(|raw| memory::parse_pressure(&raw)),
        psi_memory: read_proc("/proc/pressure/memory")
            .ok()
            .and_then(|raw| memory::parse_pressure(&raw)),
        psi_io: read_proc("/proc/pressure/io")
            .ok()
            .and_then(|raw| memory::parse_pressure(&raw)),
        pgmajfault: CountRate::per_second(
            memory::parse_pgmajfault(&vmstat_after)
                .saturating_sub(memory::parse_pgmajfault(&vmstat_before)),
            elapsed,
        ),
        net_rx: net.rx_bytes_per_sec,
        net_tx: net.tx_bytes_per_sec,
        net_rx_dropped_delta: net.rx_dropped_delta,
        net_rx_errors_delta: net.rx_errors_delta,
        interfaces: interface::read_interfaces(),
        tcp: tcp::tcp_snapshot(&snmp_before, &snmp_after, elapsed),
        temps: temp::read_temps(),
        fans: fan::read_fans(),
        disks: disk::parse_df(&df_output),
        disk_io: disk_io::parse_disk_io(
            &disk_io::parse_disk_stats(&disk_before),
            &disk_io::parse_disk_stats(&disk_after),
            elapsed,
        ),
        raid: raid::parse_mdstat(&mdstat),
        uptime: memory::parse_uptime(&uptime).unwrap_or_default(),
        sensors: portail_core::config::Config::global()
            .system_sensors
            .clone(),
    })
}
