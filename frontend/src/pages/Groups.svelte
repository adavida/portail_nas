<script lang="ts">
  import { onMount } from "svelte";
  import { listGroups, type Group } from "../api/groups";
  import { t, translate } from "../i18n";
  import GroupsTable from "../components/GroupsTable.svelte";
  import { toast } from "../components/toast";

  let groups = $state<Group[] | null>(null);
  let error = $state<string | null>(null);

  function fetchGroups() {
    listGroups()
      .then((g) => (groups = g))
      .catch(() => {
        error = "error";
        toast(translate("groups.error_loading"), true);
      });
  }

  onMount(fetchGroups);
</script>

<section>
  <h2>{$t("nav.groups")}</h2>
  {#if error}
    <p data-testid="groups-error">{$t("common.error", { error: error ?? "" })}</p>
  {:else if groups === null}
    <p data-testid="groups-loading">{$t("common.loading")}</p>
  {:else}
    <div class="card">
      <GroupsTable {groups} onUpdated={fetchGroups} onDeleted={fetchGroups} />
    </div>
  {/if}
</section>
