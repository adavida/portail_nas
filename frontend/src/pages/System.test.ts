import { render, screen } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import { mockFetchByUrl } from "../test/mock_fetch_by_url";
import System from "./System.svelte";

const stats = {
  ram_total: "16 GiB",
  ram_used: "4 GiB",
  ram_cached: "1 GiB",
  ram_buffers: "512 MiB",
  ram_dirty: "8 MiB",
  swap_total: "4 GiB",
  swap_used: "1 GiB",
  swap_cached: "16 MiB",
  cpu_usage: "25 %",
  cpu_cores: 2,
  cores: [
    { id: 0, usage: "20 %" },
    { id: 1, usage: "30 %" },
  ],
  load_1: "0.50",
  load_5: "0.40",
  load_15: "0.30",
  psi_cpu: "2.50",
  psi_memory: "0.00",
  psi_io: "1.50",
  pgmajfault: 3,
  net_rx: "2 MiB/s",
  net_tx: "2 KiB/s",
  net_rx_dropped_delta: 0,
  net_rx_errors_delta: 0,
  interfaces: [
    {
      name: "eth0",
      is_up: true,
      speed: "1 Gib/s",
      rx_dropped: 2,
      rx_errors: 0,
    },
  ],
  tcp: { established: 12, retrans_segments: 1 },
  temps: [{ name: "cpu", celsius: "45.5 °C", crit_celsius: "90 °C" }],
  fans: [{ name: "case", speed: "1200 rpm" }],
  uptime: "90061 s",
  disks: [{ mount: "/data", total: "500 GiB", used: "256 GiB" }],
  disk_io: [
    {
      name: "sda",
      read: "1 MiB/s",
      write: "2 MiB/s",
      busy: "35 %",
    },
  ],
  raid: [{ name: "md0", state: "active", resync: null }],
  sensors: [
    "memory",
    "cpu",
    "cores",
    "load",
    "pressure",
    "temps",
    "fans",
    "network",
    "tcp",
    "interfaces",
    "disk",
    "diskio",
    "raid",
    "uptime",
  ],
};

test("renders loading then bars and sparklines", async () => {
  globalThis.fetch = mockFetchByUrl({ "/api/system": stats });

  render(System);

  expect(screen.getByTestId("system-loading")).toBeInTheDocument();

  const bars = await screen.findAllByTestId("usage-bar");

  expect(bars).toHaveLength(3);
  expect(screen.getAllByTestId("gauge")).toHaveLength(5);
  expect(screen.getAllByTestId("sparkline")).toHaveLength(10);
  expect(screen.getAllByTestId("sparkline-reference")).toHaveLength(1);
  expect(screen.getByTestId("system-load")).toHaveTextContent("0.50");
  expect(screen.getByTestId("system-net-down")).toHaveTextContent("2 MiB/s");
  expect(screen.getByTestId("system-net-up")).toHaveTextContent("2 KiB/s");
  expect(screen.getByTestId("system-interfaces")).toHaveTextContent("eth0");
  expect(screen.getByTestId("system-interfaces")).toHaveTextContent("1 Gib/s");
  expect(screen.getByTestId("system-temps")).toHaveTextContent("45.5 °C");
  expect(screen.getByTestId("system-fans")).toHaveTextContent("1200 rpm");
  expect(screen.getByTestId("system-raid")).toHaveTextContent("md0");
  expect(screen.getByTestId("system-raid")).toHaveTextContent("active");
  expect(screen.getByTestId("system-uptime")).toHaveTextContent("1d 1h");
});

test("hides swap gauge when no swap configured", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/api/system": { ...stats, swap_total: "0 B", swap_used: "0 B" },
  });

  render(System);

  const gauges = await screen.findAllByTestId("gauge");

  expect(gauges).toHaveLength(4);
});

test("hides optional sensor blocks when sensors are absent", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/api/system": {
      ...stats,
      psi_cpu: null,
      psi_memory: null,
      psi_io: null,
      temps: [],
      fans: [],
      interfaces: [],
      disk_io: [],
      raid: [],
    },
  });

  render(System);

  await screen.findAllByTestId("usage-bar");

  expect(screen.queryByTestId("system-pressure")).toBeNull();
  expect(screen.queryByTestId("system-temps")).toBeNull();
  expect(screen.queryByTestId("system-fans")).toBeNull();
  expect(screen.queryByTestId("system-interfaces")).toBeNull();
  expect(screen.queryByTestId("system-disk-io")).toBeNull();
  expect(screen.queryByTestId("system-raid")).toBeNull();
});

test("hides sensor blocks excluded by system_sensors config", async () => {
  globalThis.fetch = mockFetchByUrl({
    "/api/system": { ...stats, sensors: ["memory"] },
  });

  render(System);

  const gauges = await screen.findAllByTestId("gauge");

  expect(gauges).toHaveLength(2);
  expect(screen.getAllByTestId("sparkline")).toHaveLength(1);
  expect(screen.queryByTestId("system-temps")).toBeNull();
  expect(screen.queryByTestId("system-fans")).toBeNull();
  expect(screen.queryByTestId("system-load")).toBeNull();
  expect(screen.queryByTestId("system-uptime")).toBeNull();
  expect(screen.queryByTestId("system-disks")).toBeNull();
});

test("shows error when fetch fails", async () => {
  globalThis.fetch = vi.fn(() => Promise.reject(new Error("boom")));

  render(System);

  const error = await screen.findByTestId("system-error");

  expect(error).toHaveTextContent("boom");
});
