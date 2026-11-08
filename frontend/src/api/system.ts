import { request } from "./http";
import { BitsPerSecond } from "../units/bits_per_second";
import { Bytes } from "../units/bytes";
import { BytesPerSecond } from "../units/bytes_per_second";
import { Celsius } from "../units/celsius";
import { Hundredths } from "../units/hundredths";
import { Percent } from "../units/percent";
import { Rpm } from "../units/rpm";
import { Seconds } from "../units/seconds";

export type Disk = {
  mount: string;
  total: Bytes;
  used: Bytes;
};

export type Core = {
  id: number;
  usage: Percent;
};

export type Temp = {
  name: string;
  celsius: Celsius;
  crit_celsius: Celsius | null;
};

export type Fan = {
  name: string;
  speed: Rpm;
};

export type NetInterface = {
  name: string;
  is_up: boolean;
  speed: BitsPerSecond;
  rx_dropped: number;
  rx_errors: number;
};

export type DiskIo = {
  name: string;
  read: BytesPerSecond;
  write: BytesPerSecond;
  busy: Percent;
};

export type Raid = {
  name: string;
  state: string;
  resync: Hundredths | null;
};

export type TcpStats = {
  established: number;
  retrans_segments: number;
};

export type System = {
  ram_total: Bytes;
  ram_used: Bytes;
  ram_cached: Bytes;
  ram_buffers: Bytes;
  ram_dirty: Bytes;
  swap_total: Bytes;
  swap_used: Bytes;
  swap_cached: Bytes;
  cpu_usage: Percent;
  cpu_cores: number;
  cores: Core[];
  load_1: Hundredths;
  load_5: Hundredths;
  load_15: Hundredths;
  psi_cpu: Hundredths | null;
  psi_memory: Hundredths | null;
  psi_io: Hundredths | null;
  pgmajfault: number;
  net_rx: BytesPerSecond;
  net_tx: BytesPerSecond;
  net_rx_dropped_delta: number;
  net_rx_errors_delta: number;
  interfaces: NetInterface[];
  tcp: TcpStats;
  temps: Temp[];
  fans: Fan[];
  disks: Disk[];
  disk_io: DiskIo[];
  raid: Raid[];
  uptime: Seconds;
  sensors: string[];
};

type WireSystem = {
  ram_total: string;
  ram_used: string;
  ram_cached: string;
  ram_buffers: string;
  ram_dirty: string;
  swap_total: string;
  swap_used: string;
  swap_cached: string;
  cpu_usage: string;
  cpu_cores: number;
  cores: { id: number; usage: string }[];
  load_1: string;
  load_5: string;
  load_15: string;
  psi_cpu: string | null;
  psi_memory: string | null;
  psi_io: string | null;
  pgmajfault: number;
  net_rx: string;
  net_tx: string;
  net_rx_dropped_delta: number;
  net_rx_errors_delta: number;
  interfaces: {
    name: string;
    is_up: boolean;
    speed: string;
    rx_dropped: number;
    rx_errors: number;
  }[];
  tcp: TcpStats;
  temps: { name: string; celsius: string; crit_celsius: string | null }[];
  fans: { name: string; speed: string }[];
  disks: { mount: string; total: string; used: string }[];
  disk_io: { name: string; read: string; write: string; busy: string }[];
  raid: { name: string; state: string; resync: string | null }[];
  uptime: string;
  sensors: string[];
};

function optionalHundredths(value: string | null): Hundredths | null {
  return value === null ? null : new Hundredths(value);
}

function optionalCelsius(value: string | null): Celsius | null {
  return value === null ? null : new Celsius(value);
}

function toSystem(w: WireSystem): System {
  return {
    ram_total: new Bytes(w.ram_total),
    ram_used: new Bytes(w.ram_used),
    ram_cached: new Bytes(w.ram_cached),
    ram_buffers: new Bytes(w.ram_buffers),
    ram_dirty: new Bytes(w.ram_dirty),
    swap_total: new Bytes(w.swap_total),
    swap_used: new Bytes(w.swap_used),
    swap_cached: new Bytes(w.swap_cached),
    cpu_usage: new Percent(w.cpu_usage),
    cpu_cores: w.cpu_cores,
    cores: w.cores.map((core) => ({
      id: core.id,
      usage: new Percent(core.usage),
    })),
    load_1: new Hundredths(w.load_1),
    load_5: new Hundredths(w.load_5),
    load_15: new Hundredths(w.load_15),
    psi_cpu: optionalHundredths(w.psi_cpu),
    psi_memory: optionalHundredths(w.psi_memory),
    psi_io: optionalHundredths(w.psi_io),
    pgmajfault: w.pgmajfault,
    net_rx: new BytesPerSecond(w.net_rx),
    net_tx: new BytesPerSecond(w.net_tx),
    net_rx_dropped_delta: w.net_rx_dropped_delta,
    net_rx_errors_delta: w.net_rx_errors_delta,
    interfaces: w.interfaces.map((iface) => ({
      name: iface.name,
      is_up: iface.is_up,
      speed: new BitsPerSecond(iface.speed),
      rx_dropped: iface.rx_dropped,
      rx_errors: iface.rx_errors,
    })),
    tcp: w.tcp,
    temps: w.temps.map((temp) => ({
      name: temp.name,
      celsius: new Celsius(temp.celsius),
      crit_celsius: optionalCelsius(temp.crit_celsius),
    })),
    fans: w.fans.map((fan) => ({
      name: fan.name,
      speed: new Rpm(fan.speed),
    })),
    disks: w.disks.map((disk) => ({
      mount: disk.mount,
      total: new Bytes(disk.total),
      used: new Bytes(disk.used),
    })),
    disk_io: w.disk_io.map((disk) => ({
      name: disk.name,
      read: new BytesPerSecond(disk.read),
      write: new BytesPerSecond(disk.write),
      busy: new Percent(disk.busy),
    })),
    raid: w.raid.map((raid) => ({
      name: raid.name,
      state: raid.state,
      resync: optionalHundredths(raid.resync),
    })),
    uptime: new Seconds(w.uptime),
    sensors: w.sensors,
  };
}

export async function getSystem(): Promise<System> {
  const wire = await request<WireSystem>("GET", "/api/system");
  return toSystem(wire);
}
