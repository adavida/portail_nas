<script lang="ts">
  import { onMount } from "svelte";
  import { listGroups, type Group } from "../api/groups";
  import { listUsers, type User } from "../api/users";
  import UsersTable from "../components/UsersTable.svelte";
  import { toast } from "../components/toast";

  let users = $state<User[] | null>(null);
  let groups = $state<Group[]>([]);
  let error = $state<string | null>(null);

  function fetchUsers() {
    listGroups()
      .then((g) => (groups = g))
      .catch(() => toast("Erreur: chargement des groupes impossible", true));
    listUsers()
      .then((u) => (users = u))
      .catch(() => {
        error = "error";
        toast("Erreur: chargement des utilisateurs impossible", true);
      });
  }

  onMount(fetchUsers);
</script>

<section>
  <h2>Utilisateurs</h2>
  {#if error}
    <p data-testid="users-error">Erreur: {error}</p>
  {:else if users === null}
    <p data-testid="users-loading">Chargement...</p>
  {:else}
    <div class="card">
      <UsersTable
        {users}
        allGroups={groups.map((g) => g.gid)}
        onCreated={fetchUsers}
        onUpdated={fetchUsers}
        onDeleted={fetchUsers}
      />
    </div>
  {/if}
</section>
