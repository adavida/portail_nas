<script lang="ts">
  import { onMount } from "svelte";
  import { listGroups, type Group } from "../api/groups";
  import { listUsers, type User } from "../api/users";
  import { t, translate } from "../i18n";
  import UsersTable from "../components/UsersTable.svelte";
  import { toast } from "../components/toast";

  let users = $state<User[] | null>(null);
  let groups = $state<Group[]>([]);
  let error = $state<string | null>(null);

  function fetchUsers() {
    listGroups()
      .then((g) => (groups = g))
      .catch(() => toast(translate("groups.error_loading"), true));
    listUsers()
      .then((u) => (users = u))
      .catch(() => {
        error = "error";
        toast(translate("users.error_loading"), true);
      });
  }

  onMount(fetchUsers);
</script>

<section>
  <h2>{$t("nav.users")}</h2>
  {#if error}
    <p data-testid="users-error">{$t("common.error", { error: error ?? "" })}</p>
  {:else if users === null}
    <p data-testid="users-loading">{$t("common.loading")}</p>
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
