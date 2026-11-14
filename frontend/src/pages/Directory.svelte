<script lang="ts">
  import { t } from "../i18n";
  import Tabs from "../components/Tabs.svelte";
  import Groups from "./Groups.svelte";
  import Users from "./Users.svelte";

  let tab = $state("users");
</script>

<section>
  <Tabs
    tabs={[
      { id: "users", label: $t("nav.users"), testId: "directory-tab-users" },
      { id: "groups", label: $t("nav.groups"), testId: "directory-tab-groups" },
    ]}
    bind:active={tab}
  />
  {#if tab === "users"}
    <Users />
  {:else}
    <Groups />
  {/if}
</section>
