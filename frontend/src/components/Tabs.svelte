<script lang="ts">
  import "./Tabs.scss";

  type Tab = { id: string; label: string; testId: string };

  type Props = {
    tabs: Tab[];
    active: string;
  };

  let { tabs, active = $bindable() }: Props = $props();
</script>

<div class="tabs" role="tablist">
  {#each tabs as tab (tab.id)}
    <button
      type="button"
      role="tab"
      aria-selected={active === tab.id}
      class={active === tab.id ? "active" : ""}
      data-testid={tab.testId}
      onclick={() => (active = tab.id)}
    >
      {tab.label}
    </button>
  {/each}
</div>
