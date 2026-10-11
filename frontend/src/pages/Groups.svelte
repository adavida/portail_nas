<script lang="ts">
  import { onMount } from "svelte";
  import { listGroups, type Group } from "../api/groups";
  import GroupsTable from "../components/GroupsTable.svelte";
  import { toast } from "../components/toast";

  let groups = $state<Group[] | null>(null);
  let error = $state<string | null>(null);

  function fetchGroups() {
    listGroups()
      .then((g) => (groups = g))
      .catch(() => {
        error = "error";
        toast("Erreur: chargement des groupes impossible", true);
      });
  }

  onMount(fetchGroups);
</script>

<section>
  <h2>Groupes</h2>
  {#if error}
    <p data-testid="groups-error">Erreur: {error}</p>
  {:else if groups === null}
    <p data-testid="groups-loading">Chargement...</p>
  {:else}
    <div class="card">
      <GroupsTable {groups} onUpdated={fetchGroups} onDeleted={fetchGroups} />
    </div>
  {/if}
</section>
