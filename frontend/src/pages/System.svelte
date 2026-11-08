<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { getSystem, type System, type Temp } from "../api/system";
  import type { Bytes } from "../units/bytes";
  import Gauge from "../components/Gauge.svelte";
  import Sparkline from "../components/Sparkline.svelte";
  import UsageBar from "../components/UsageBar.svelte";
  import { toast } from "../components/toast";
  import { t, translate } from "../i18n";

  const REFRESH_INTERVAL_MS = 5000;
  const HISTORY_MAX_POINTS = 60;
  const BYTES_PER_KIBIBYTE = 1024;
  const LOAD_MAX_FACTOR = 2;
  const TEMP_MAX_CELSIUS = 100;
  const FAN_MAX_RPM = 7000;
  const SECS_PER_DAY = 86400;
  const SECS_PER_HOUR = 3600;

  let stats = $state<System | null>(null);
  let error = $state<string | null>(null);
  let timer: ReturnType<typeof setInterval> | undefined;
  let ramHistory = $state<number[]>([]);
  let swapHistory = $state<number[]>([]);
  let cpuHistory = $state<number[]>([]);
  let loadHistory = $state<number[]>([]);
  let netRxHistory = $state<number[]>([]);
  let netTxHistory = $state<number[]>([]);
  let psiCpuHistory = $state<number[]>([]);
  let psiMemoryHistory = $state<number[]>([]);
  let psiIoHistory = $state<number[]>([]);
  let faultHistory = $state<number[]>([]);
  let dropsHistory = $state<number[]>([]);
  let establishedHistory = $state<number[]>([]);
  let retransHistory = $state<number[]>([]);
  let tempHistory = $state<Record<string, number[]>>({});
  let diskBusyHistory = $state<Record<string, number[]>>({});

  const netMax = $derived(Math.max(1, ...netRxHistory, ...netTxHistory));
  const faultMax = $derived(Math.max(1, ...faultHistory));
  const dropsMax = $derived(Math.max(1, ...dropsHistory));
  const tcpMax = $derived(
    Math.max(1, ...establishedHistory, ...retransHistory),
  );

  function pushSample(history: number[], value: number) {
    history.push(value);
    if (history.length > HISTORY_MAX_POINTS) {
      history.splice(0, history.length - HISTORY_MAX_POINTS);
    }
  }

  function pushSeries(
    map: Record<string, number[]>,
    key: string,
    value: number,
  ) {
    const existing = map[key];
    if (existing === undefined) {
      map[key] = [value];
    } else {
      pushSample(existing, value);
    }
  }

  function percentOf(used: Bytes, total: Bytes): number {
    return total.value === 0 ? 0 : Math.round((used.value * 100) / total.value);
  }

  function tempPercent(temp: Temp) {
    return temp.crit_celsius === null
      ? 0
      : Math.round((temp.celsius.value * 100) / temp.crit_celsius.value);
  }

  function fetchStats() {
    getSystem()
      .then((s) => {
        stats = s;
        error = null;
        pushSample(ramHistory, percentOf(s.ram_used, s.ram_total));
        pushSample(swapHistory, percentOf(s.swap_used, s.swap_total));
        pushSample(cpuHistory, s.cpu_usage.value);
        pushSample(loadHistory, s.load_1.value);
        pushSample(netRxHistory, s.net_rx.value / BYTES_PER_KIBIBYTE);
        pushSample(netTxHistory, s.net_tx.value / BYTES_PER_KIBIBYTE);
        pushSample(psiCpuHistory, s.psi_cpu?.value ?? 0);
        pushSample(psiMemoryHistory, s.psi_memory?.value ?? 0);
        pushSample(psiIoHistory, s.psi_io?.value ?? 0);
        pushSample(faultHistory, s.pgmajfault);
        pushSample(dropsHistory, s.net_rx_dropped_delta);
        pushSample(establishedHistory, s.tcp.established);
        pushSample(retransHistory, s.tcp.retrans_segments);
        for (const temp of s.temps) {
          pushSeries(tempHistory, temp.name, temp.celsius.value);
        }
        for (const disk of s.disk_io) {
          pushSeries(diskBusyHistory, disk.name, disk.busy.value);
        }
      })
      .catch((err: Error) => {
        error = err.message;
        toast(translate("system.error_loading"), true);
      });
  }

  function shown(sensor: string) {
    return stats !== null && stats.sensors.includes(sensor);
  }

  function formatUptime(secs: number) {
    const days = Math.floor(secs / SECS_PER_DAY);
    const hours = Math.floor((secs % SECS_PER_DAY) / SECS_PER_HOUR);
    return days > 0 ? `${days}d ${hours}h` : `${hours}h`;
  }

  onMount(() => {
    fetchStats();
    timer = setInterval(fetchStats, REFRESH_INTERVAL_MS);
  });

  onDestroy(() => clearInterval(timer));
</script>

<section>
  <h2>{$t("nav.system")}</h2>
  {#if stats === null && error}
    <p data-testid="system-error">{$t("common.error", { error })}</p>
  {:else if stats === null}
    <p data-testid="system-loading">{$t("common.loading")}</p>
  {:else}
    <div class="card system-grid">
      {#if shown("memory")}
        <div class="system-usage">
          <div class="gauge-row">
            <Gauge
              label={$t("system.ram")}
              percent={percentOf(stats.ram_used, stats.ram_total)}
              detail={`${stats.ram_used} / ${stats.ram_total}`}
            />
            {#if stats.swap_total.value > 0}
              <Gauge
                label={$t("system.swap")}
                percent={percentOf(stats.swap_used, stats.swap_total)}
                detail={`${stats.swap_used} / ${stats.swap_total}`}
              />
            {/if}
          </div>
          <Sparkline
            series={[
              { label: $t("system.ram"), points: ramHistory },
              ...(stats.swap_total.value > 0
                ? [{ label: $t("system.swap"), points: swapHistory }]
                : []),
            ]}
            unit="%"
          />
        </div>
      {/if}
      {#if shown("cpu")}
        <div class="system-usage">
          <UsageBar
            label={$t("system.cpu")}
            percent={stats.cpu_usage.value}
            detail={`${stats.cpu_usage} · ${stats.cpu_cores} ${$t("system.cores")}`}
          />
          <Sparkline series={[{ label: "", points: cpuHistory }]} unit="%" />
        </div>
      {/if}
      {#if shown("cores")}
        <div class="gauge-row" data-testid="system-cores">
          {#each stats.cores as core (core.id)}
            <div class="core">
              <UsageBar label={`c${core.id}`} percent={core.usage.value} />
            </div>
          {/each}
        </div>
      {/if}
      {#if shown("load")}
        <div class="system-usage">
          <div class="system-line" data-testid="system-load">
            <span>{$t("system.load")}</span>
            <span
              >{stats.load_1} · {stats.load_5} · {stats.load_15}</span
            >
          </div>
          <Sparkline
            series={[{ label: "", points: loadHistory }]}
            max={stats.cpu_cores * LOAD_MAX_FACTOR}
            reference={stats.cpu_cores}
          />
        </div>
      {/if}
      {#if shown("pressure") && stats.psi_cpu !== null}
        <div class="system-usage">
          <div class="system-line" data-testid="system-pressure">
            <span>{$t("system.pressure")}</span>
            <span
              >{stats.psi_cpu} · {stats.psi_memory ?? "0"} ·
              {stats.psi_io ?? "0"}</span
            >
          </div>
          <Sparkline
            series={[
              { label: $t("system.pressure_cpu"), points: psiCpuHistory },
              { label: $t("system.pressure_memory"), points: psiMemoryHistory },
              { label: $t("system.pressure_io"), points: psiIoHistory },
            ]}
            unit="%"
          />
        </div>
      {/if}
      {#if shown("pressure")}
        <div class="system-usage">
          <div class="system-line" data-testid="system-faults">
            <span>{$t("system.faults")}</span>
            <span>{stats.pgmajfault} /s</span>
          </div>
          <Sparkline
            series={[{ label: "", points: faultHistory }]}
            max={faultMax}
            unit="/s"
          />
        </div>
      {/if}
      {#if shown("temps") && stats.temps.length > 0}
        <div class="system-usage">
          <div class="gauge-row" data-testid="system-temps">
            {#each stats.temps as temp (temp.name)}
              <Gauge
                label={temp.name}
                percent={tempPercent(temp)}
                detail={`${temp.celsius}`}
              />
            {/each}
          </div>
          <Sparkline
            series={stats.temps.map((temp) => ({
              label: temp.name,
              points: tempHistory[temp.name] ?? [],
            }))}
            max={TEMP_MAX_CELSIUS}
            unit="°C"
          />
        </div>
      {/if}
      {#if shown("fans") && stats.fans.length > 0}
        <div class="gauge-row" data-testid="system-fans">
          {#each stats.fans as fan (fan.name)}
            <Gauge
              label={fan.name}
              percent={(fan.speed.value * 100) / FAN_MAX_RPM}
              detail={`${fan.speed}`}
            />
          {/each}
        </div>
      {/if}
      {#if shown("network")}
        <div class="system-usage">
          <div class="system-line">
            <span data-testid="system-net-down">
              {$t("system.net_down")} {stats.net_rx}
            </span>
            <span data-testid="system-net-up">
              {$t("system.net_up")} {stats.net_tx}
            </span>
          </div>
          <Sparkline
            series={[
              { label: $t("system.net_down"), points: netRxHistory },
              { label: $t("system.net_up"), points: netTxHistory },
            ]}
            max={netMax}
            unit="KiB/s"
          />
        </div>
        <div class="system-usage">
          <div class="system-line" data-testid="system-net-delta">
            <span>{$t("system.delta")}</span>
            <span
              >{$t("system.drops")} {stats.net_rx_dropped_delta} ·
              {$t("system.errors")} {stats.net_rx_errors_delta}</span
            >
          </div>
          <Sparkline
            series={[{ label: "", points: dropsHistory }]}
            max={dropsMax}
          />
        </div>
      {/if}
      {#if shown("tcp")}
        <div class="system-usage">
          <div class="system-line" data-testid="system-tcp">
            <span>{$t("system.tcp")}</span>
            <span
              >{$t("system.sessions")} {stats.tcp.established} ·
              {$t("system.retrans")} {stats.tcp.retrans_segments}</span
            >
          </div>
          <Sparkline
            series={[
              { label: $t("system.sessions"), points: establishedHistory },
              { label: $t("system.retrans"), points: retransHistory },
            ]}
            max={tcpMax}
          />
        </div>
      {/if}
      {#if shown("interfaces") && stats.interfaces.length > 0}
        <div data-testid="system-interfaces">
          {#each stats.interfaces as iface (iface.name)}
            <div class="system-line">
              <span>{iface.name}</span>
              <span
                >{iface.is_up ? "up" : "down"}{#if iface.speed.value > 0}
                  · {iface.speed}{/if}
                  · {$t("system.drops")} {iface.rx_dropped} ·
                  {$t("system.errors")} {iface.rx_errors}</span
              >
            </div>
          {/each}
        </div>
      {/if}
      {#if shown("uptime")}
        <div class="system-line" data-testid="system-uptime">
          <span>{$t("system.uptime")}</span>
          <span>{formatUptime(stats.uptime.value)}</span>
        </div>
      {/if}
      {#if shown("disk")}
        <div class="gauge-row" data-testid="system-disks">
          {#each stats.disks as disk (disk.mount)}
            <Gauge
              label={`${$t("system.disk")} ${disk.mount}`}
              percent={percentOf(disk.used, disk.total)}
              detail={`${disk.used} / ${disk.total}`}
            />
          {/each}
        </div>
      {/if}
      {#if shown("diskio") && stats.disk_io.length > 0}
        <div class="system-usage">
          <div data-testid="system-disk-io">
            {#each stats.disk_io as disk (disk.name)}
              <div class="system-line">
                <span>{disk.name}</span>
                <span
                  >{disk.read} ↓ · {disk.write} ↑</span
                >
              </div>
            {/each}
          </div>
          <Sparkline
            series={stats.disk_io.map((disk) => ({
              label: disk.name,
              points: diskBusyHistory[disk.name] ?? [],
            }))}
            unit="%"
          />
        </div>
      {/if}
      {#if shown("raid") && stats.raid.length > 0}
        <div data-testid="system-raid">
          {#each stats.raid as raid (raid.name)}
            <div class="system-line">
              <span>{$t("system.raid")} {raid.name}</span>
              <span
                >{raid.state}{#if raid.resync !== null}
                  · {raid.resync} %{/if}</span
              >
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</section>

<style>
  .system-grid {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .gauge-row {
    display: flex;
    flex-direction: row;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .core {
    width: 6rem;
  }

  .system-usage {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .system-line {
    display: flex;
    justify-content: space-between;
    font-size: 0.9rem;
  }
</style>
