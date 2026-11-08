<script lang="ts">
  let {
    label,
    percent,
    detail = "",
  }: { label: string; percent: number; detail?: string } = $props();

  const clamped = $derived(Math.min(100, Math.max(0, percent)));
  const needle = $derived.by(() => {
    const angle = (Math.PI * clamped) / 100;
    return {
      x: (50 - 40 * Math.cos(angle)).toFixed(2),
      y: (50 - 40 * Math.sin(angle)).toFixed(2),
    };
  });
</script>

<div class="gauge" data-testid="gauge">
  <svg
    class="dial"
    viewBox="0 0 100 55"
    role="img"
    aria-label={detail === "" ? label : `${label}: ${detail}`}
  >
    <path class="track" d="M10 50 A40 40 0 0 1 90 50" pathLength="100" />
    <path
      class="redzone"
      d="M10 50 A40 40 0 0 1 90 50"
      pathLength="100"
      stroke-dasharray="10 100"
      stroke-dashoffset="-90"
    />
    <path
      class="fill"
      class:hot={clamped >= 90}
      data-testid="gauge-fill"
      d="M10 50 A40 40 0 0 1 90 50"
      pathLength="100"
      stroke-dasharray={`${clamped} 100`}
    />
    <line
      data-testid="gauge-needle"
      x1="50"
      y1="50"
      x2={needle.x}
      y2={needle.y}
    />
    <circle cx="50" cy="50" r="3" />
  </svg>
  <div class="gauge-label">{label}</div>
  <div class="gauge-detail">{detail}</div>
</div>

<style>
  .gauge {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    width: 10rem;
  }
  .dial {
    width: 100%;
  }
  .track,
  .redzone,
  .fill {
    fill: none;
    stroke-linecap: round;
  }
  .track {
    stroke: color-mix(in srgb, currentColor 15%, transparent);
    stroke-width: 7;
  }
  .redzone {
    stroke: #e5484d;
    stroke-width: 7;
    opacity: 0.4;
  }
  .fill {
    stroke: var(--accent, #4f7cff);
    stroke-width: 7;
  }
  .fill.hot {
    stroke: #e5484d;
  }
  line {
    stroke: currentColor;
    stroke-width: 2;
  }
  circle {
    fill: currentColor;
  }
  .gauge-label {
    font-size: 0.9rem;
  }
  .gauge-detail {
    font-size: 0.8rem;
    opacity: 0.7;
  }
</style>
