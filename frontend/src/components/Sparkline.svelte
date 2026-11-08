<script lang="ts">
  type Series = { label: string; points: number[]; color?: string };

  let {
    series = [],
    height = 60,
    max = 100,
    unit = "",
    reference = null,
  }: {
    series?: Series[];
    height?: number;
    max?: number;
    unit?: string;
    reference?: number | null;
  } = $props();

  const PALETTE = [
    "var(--accent, #4f7cff)",
    "#f5a623",
    "#3ba95c",
    "#e5484d",
    "#a78bfa",
  ];

  function toY(value: number) {
    return 100 - Math.min(100, Math.max(0, (value * 100) / max));
  }

  function formatTick(value: number) {
    const tick = Math.round(value * 10) / 10;
    return unit === "" ? `${tick}` : `${tick} ${unit}`;
  }

  function toCoords(values: number[]) {
    return values.length >= 2
      ? values
          .map((value, i) => {
            const x = (i / (values.length - 1)) * 100;
            return `${x.toFixed(2)},${toY(value).toFixed(2)}`;
          })
          .join(" ")
      : "";
  }

  const lines = $derived(
    series
      .map((entry, index) => ({
        label: entry.label,
        color: entry.color ?? PALETTE[index % PALETTE.length],
        coords: toCoords(entry.points),
      }))
      .filter((entry) => entry.coords !== ""),
  );
  const legend = $derived(
    lines.length > 1 ? lines.filter((entry) => entry.label !== "") : [],
  );
  const referenceY = $derived(reference === null ? null : toY(reference));
</script>

<div class="spark-wrap">
  <div class="spark-chart">
    <svg
      class="spark"
      viewBox="0 0 100 100"
      preserveAspectRatio="none"
      role="img"
      aria-hidden="true"
      data-testid="sparkline"
      style:height="{height}px"
    >
      {#each lines as line (line.label + line.color)}
        <polyline
          points={line.coords}
          fill="none"
          stroke={line.color}
          stroke-width="1.5"
          vector-effect="non-scaling-stroke"
        />
      {/each}
      {#if referenceY !== null}
        <line
          data-testid="sparkline-reference"
          x1="0"
          x2="100"
          y1={referenceY}
          y2={referenceY}
          stroke="currentColor"
          stroke-dasharray="4 4"
          stroke-width="1"
          vector-effect="non-scaling-stroke"
          opacity="0.4"
        />
      {/if}
    </svg>
    <div class="spark-axis" data-testid="sparkline-axis">
      <span>{formatTick(max)}</span>
      <span>{formatTick(0)}</span>
    </div>
  </div>
  {#if legend.length > 0}
    <div class="legend" data-testid="sparkline-legend">
      {#each legend as entry (entry.label)}
        <span class="legend-item">
          <span class="swatch" style:background={entry.color}></span>
          {entry.label}
        </span>
      {/each}
    </div>
  {/if}
</div>

<style>
  .spark-wrap {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .spark-chart {
    display: flex;
    gap: 0.35rem;
  }
  .spark-axis {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    font-size: 0.7rem;
    opacity: 0.6;
    white-space: nowrap;
  }
  .spark {
    display: block;
    flex: 1;
    min-width: 0;
    opacity: 0.8;
  }
  .legend {
    display: flex;
    gap: 1rem;
    font-size: 0.8rem;
    opacity: 0.8;
  }
  .legend-item {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }
  .swatch {
    width: 0.75rem;
    height: 0.25rem;
    border-radius: 999px;
  }
</style>
