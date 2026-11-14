<script lang="ts">
  import { t } from "../i18n";
  import Tabs from "../components/Tabs.svelte";
  import Debug from "./Debug.svelte";
  import System from "./System.svelte";

  let tab = $state("system");
</script>

<section>
  <Tabs
    tabs={[
      {
        id: "system",
        label: $t("nav.system"),
        testId: "monitoring-tab-system",
      },
      { id: "debug", label: $t("nav.diagnostics"), testId: "monitoring-tab-debug" },
    ]}
    bind:active={tab}
  />
  {#if tab === "system"}
    <System />
  {:else}
    <Debug />
  {/if}
</section>
