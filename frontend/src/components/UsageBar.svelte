<script lang="ts">
  let {
    label,
    percent,
    detail = "",
  }: { label: string; percent: number; detail?: string } = $props();

  const clamped = $derived(Math.min(100, Math.max(0, percent)));
</script>

<div class="usage" data-testid="usage-bar">
  <div class="usage-head">
    <span class="usage-label">{label}</span>
    <span class="usage-detail">{detail}</span>
  </div>
  <div class="usage-track">
    <div class="usage-fill" data-testid="usage-fill" style="width: {clamped}%"></div>
  </div>
</div>

<style>
  .usage {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .usage-head {
    display: flex;
    justify-content: space-between;
    font-size: 0.9rem;
  }
  .usage-detail {
    opacity: 0.7;
  }
  .usage-track {
    height: 0.5rem;
    border-radius: 999px;
    background: color-mix(in srgb, currentColor 15%, transparent);
    overflow: hidden;
  }
  .usage-fill {
    height: 100%;
    border-radius: 999px;
    background: var(--accent, #4f7cff);
  }
</style>
