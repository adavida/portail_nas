<script lang="ts">
  import { deleteUser, updatePassword, updateUser, type User } from "../api/users";
  import { addMember, createGroup, removeMember } from "../api/groups";
  import EditableCell from "./EditableCell.svelte";
  import GroupsEditor from "./GroupsEditor.svelte";
  import UserModal from "./UserModal.svelte";
  import UserCreateModal from "./UserCreateModal.svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import { t, translate } from "../i18n";
  import { toast } from "./toast";

  type Props = {
    users: User[];
    allGroups?: string[];
    onCreated?: () => void;
    onUpdated?: () => void;
    onDeleted?: () => void;
  };

  // $bp-narrow des tokens — doit rester synchronisé
  const MOBILE_QUERY = "(max-width: 720px)";

  let { users, allGroups = [], onCreated, onUpdated, onDeleted }: Props =
    $props();

  let mobile = $state(false);

  $effect(() => {
    const mq = window.matchMedia(MOBILE_QUERY);
    const sync = () => (mobile = mq.matches);
    sync();
    mq.addEventListener("change", sync);
    return () => mq.removeEventListener("change", sync);
  });

  let editingGroupsUid = $state<string | null>(null);
  let editingUid = $state<string | null>(null);
  let deleteTarget = $state<User | null>(null);
  const editingUser = $derived(
    editingUid === null ? null : users.find((u) => u.uid === editingUid),
  );
  let passwords = $state<Record<string, string>>({});
  let creating = $state(false);

  async function saveGroups(user: User, next: string[]): Promise<boolean> {
    const groups = user.groups ?? [];
    const adds = next.filter((gid) => !groups.includes(gid));
    const removes = groups.filter((gid) => !next.includes(gid));
    try {
      for (const gid of adds) await addMember(gid, user.uid);
      for (const gid of removes) await removeMember(gid, user.uid);
    } catch (e) {
      toast((e as Error).message, true);
      return false;
    }
    toast(translate("common.saved"));
    onUpdated?.();
    return true;
  }

  async function createGroupWithMember(
    user: User,
    gid: string,
  ): Promise<{ ok: boolean; error?: string }> {
    try {
      await createGroup({
        gid,
        name: gid,
        description: "",
        members: [user.uid],
      });
    } catch (e) {
      const error = (e as Error).message;
      toast(error, true);
      return { ok: false, error };
    }
    toast(translate("groups.created", { gid }));
    onUpdated?.();
    return { ok: true };
  }

  async function submitPassword(user: User) {
    const password = passwords[user.uid] ?? "";
    if (!password) {
      toast(translate("common.password_required"), true);
      return;
    }
    try {
      await updatePassword(user.uid, password);
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    passwords[user.uid] = "";
    toast(translate("common.password_changed"));
  }

  async function removeUser(user: User) {
    try {
      await deleteUser(user.uid);
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    onDeleted?.();
    toast(translate("users.deleted", { uid: user.uid }));
  }

  async function saveField(
    user: User,
    field: string,
    newValue: string,
  ): Promise<boolean> {
    const body =
      field === "name"
        ? { name: newValue, email: user.email }
        : { name: user.name, email: newValue };
    try {
      await updateUser(user.uid, body);
    } catch (e) {
      toast((e as Error).message, true);
      return false;
    }
    onUpdated?.();
    return true;
  }
</script>

{#if onCreated}
  <div class="row-actions create-bar">
    <button
      class="btn btn-primary"
      type="button"
      data-testid="create-user-button"
      onclick={() => (creating = true)}
    >
      {$t("users.create")}
    </button>
  </div>
{/if}

{#if mobile}
  <div class="table-scroll">
    <table data-testid="users-table" class="table">
      <thead>
        <tr>
          <th>UID</th>
          <th>{$t("col.name")}</th>
          <th>{$t("col.email")}</th>
          <th>{$t("col.groups")}</th>
        </tr>
      </thead>
      <tbody>
        {#if users.length === 0}
          <tr>
            <td colSpan={4} class="center" data-testid="users-empty">
              {$t("users.empty")}
            </td>
          </tr>
        {:else}
          {#each users as u (u.uid)}
            {@const visibleGroups = (u.groups ?? []).filter((g) => g !== "user")}
            <tr
              data-testid={`user-row-${u.uid}`}
              class="clickable"
              tabindex="0"
              onclick={() => (editingUid = u.uid)}
              onkeydown={(e) => e.key === "Enter" && (editingUid = u.uid)}
            >
              <td>{u.uid}</td>
              <td>{u.name}</td>
              <td>{u.email}</td>
              <td data-testid={`cell-groups-${u.uid}`}>
                {visibleGroups.length ? visibleGroups.join(" ") : "-"}
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>

  {#if editingUser}
    <UserModal
      user={editingUser}
      {allGroups}
      onClose={() => (editingUid = null)}
      {onUpdated}
      {onDeleted}
    />
  {/if}
{:else}
  <div class="table-scroll">
    <table data-testid="users-table" class="table">
      <thead>
        <tr>
          <th>UID</th>
          <th>{$t("col.name")}</th>
          <th>{$t("col.email")}</th>
          <th>{$t("col.groups")}</th>
          <th>{$t("col.password")}</th>
          <th>{$t("col.action")}</th>
        </tr>
      </thead>
      <tbody>
        {#if users.length === 0}
          <tr>
            <td colSpan={6} class="center" data-testid="users-empty">
              {$t("users.empty")}
            </td>
          </tr>
        {:else}
          {#each users as u (u.uid)}
            {@const groups = u.groups ?? []}
            {@const visibleGroups = groups.filter((g) => g !== "user")}
            <tr data-testid={`user-row-${u.uid}`}>
              <td>{u.uid}</td>
              <EditableCell
                value={u.name}
                field="name"
                rowId={u.uid}
                onSave={(f, v) => saveField(u, f, v)}
              />
              <EditableCell
                value={u.email}
                field="email"
                rowId={u.uid}
                onSave={(f, v) => saveField(u, f, v)}
              />
              {#if editingGroupsUid !== u.uid}
                <td
                  data-testid={`cell-groups-${u.uid}`}
                  class="editable"
                  onclick={() => (editingGroupsUid = u.uid)}
                >
                  {visibleGroups.length ? visibleGroups.join(" ") : "-"}
                </td>
              {:else}
                <td>
                  <GroupsEditor
                    uid={u.uid}
                    {groups}
                    allGroups={allGroups}
                    onSave={(next) => saveGroups(u, next)}
                    onCreateGroup={(gid) => createGroupWithMember(u, gid)}
                    onCancel={() => (editingGroupsUid = null)}
                    hideActions
                  />
                </td>
              {/if}
              <td>
                <span class="row-actions">
                  <input
                    class="input"
                    data-testid={`password-input-${u.uid}`}
                    placeholder={$t("common.password_new")}
                    type="password"
                    bind:value={passwords[u.uid]}
                  />
                  <button
                    class="btn btn-primary"
                    data-testid={`password-button-${u.uid}`}
                    onclick={() => void submitPassword(u)}
                    type="button"
                  >
                    {$t("common.edit")}
                  </button>
                </span>
              </td>
              <td>
                <button
                  class="btn btn-danger"
                  data-testid={`delete-button-${u.uid}`}
                  onclick={() => (deleteTarget = u)}
                  type="button"
                >
                  {$t("common.delete")}
                </button>
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
{/if}

{#if deleteTarget}
  <ConfirmModal
    message={$t("users.confirm_delete", { uid: deleteTarget.uid })}
    onConfirm={() => {
      const target = deleteTarget;
      deleteTarget = null;
      if (target) void removeUser(target);
    }}
    onCancel={() => (deleteTarget = null)}
  />
{/if}

{#if creating}
  <UserCreateModal onClose={() => (creating = false)} {onCreated} />
{/if}
