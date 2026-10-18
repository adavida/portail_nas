<script lang="ts">
  import { deleteGroup, updateGroup, type Group } from "../api/groups";
  import EditableCell from "./EditableCell.svelte";
  import GroupModal from "./GroupModal.svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import { t, translate } from "../i18n";
  import { toast } from "./toast";

  type Props = {
    groups: Group[];
    onUpdated?: () => void;
    onDeleted?: () => void;
  };

  // $bp-narrow from tokens — must stay in sync
  const MOBILE_QUERY = "(max-width: 720px)";
  const PROTECTED_GIDS = ["admin", "user"];
  const isProtected = (gid: string) => PROTECTED_GIDS.includes(gid);
  const isHidden = (gid: string) => gid === "user";

  let { groups, onUpdated, onDeleted }: Props = $props();

  const visible = $derived(groups.filter((g) => !isHidden(g.gid)));

  let mobile = $state(false);
  let editingGid = $state<string | null>(null);
  let deleteTarget = $state<Group | null>(null);
  const editingGroup = $derived(
    editingGid === null ? null : groups.find((g) => g.gid === editingGid),
  );

  $effect(() => {
    const mq = window.matchMedia(MOBILE_QUERY);
    const sync = () => (mobile = mq.matches);
    sync();
    mq.addEventListener("change", sync);
    return () => mq.removeEventListener("change", sync);
  });

  async function remove(group: Group) {
    try {
      await deleteGroup(group.gid);
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    onDeleted?.();
    toast(translate("groups.deleted", { gid: group.gid }));
  }

  async function saveField(
    group: Group,
    field: string,
    newValue: string,
  ): Promise<boolean> {
    const body =
      field === "name"
        ? { name: newValue, description: group.description }
        : { name: group.name, description: newValue };
    try {
      await updateGroup(group.gid, body);
    } catch (e) {
      toast((e as Error).message, true);
      return false;
    }
    toast(translate("common.saved"));
    onUpdated?.();
    return true;
  }
</script>

{#if mobile}
  <div class="table-scroll">
    <table data-testid="groups-table" class="table">
      <thead>
        <tr>
          <th>GID</th>
          <th>{$t("col.name")}</th>
          <th>{$t("col.description")}</th>
        </tr>
      </thead>
      <tbody>
        {#if visible.length === 0}
          <tr>
            <td colSpan={3} class="center" data-testid="groups-empty">
              {$t("groups.empty")}
            </td>
          </tr>
        {:else}
          {#each visible as g (g.gid)}
            <tr
              data-testid={`group-row-${g.gid}`}
              class="clickable"
              tabindex="0"
              onclick={() => (editingGid = g.gid)}
              onkeydown={(e) => e.key === "Enter" && (editingGid = g.gid)}
            >
              <td>{g.gid}</td>
              <td>{g.name}</td>
              <td>{g.description}</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>

  {#if editingGroup}
    <GroupModal
      group={editingGroup}
      canDelete={!isProtected(editingGroup.gid)}
      onClose={() => (editingGid = null)}
      {onUpdated}
      {onDeleted}
    />
  {/if}
{:else}
  <div class="table-scroll">
    <table data-testid="groups-table" class="table">
      <thead>
        <tr>
          <th>GID</th>
          <th>{$t("col.name")}</th>
          <th>{$t("col.description")}</th>
          <th>{$t("col.action")}</th>
        </tr>
      </thead>
      <tbody>
        {#if visible.length === 0}
          <tr>
            <td colSpan={4} class="center" data-testid="groups-empty">
              {$t("groups.empty")}
            </td>
          </tr>
        {:else}
          {#each visible as g (g.gid)}
            <tr data-testid={`group-row-${g.gid}`}>
              <td>{g.gid}</td>
              <EditableCell
                value={g.name}
                field="name"
                rowId={g.gid}
                onSave={(f, v) => saveField(g, f, v)}
              />
              <EditableCell
                value={g.description}
                field="description"
                rowId={g.gid}
                onSave={(f, v) => saveField(g, f, v)}
              />
              <td>
                {#if !isProtected(g.gid)}
                  <button
                    class="btn btn-danger"
                    data-testid={`group-delete-button-${g.gid}`}
                    onclick={() => (deleteTarget = g)}
                    type="button"
                  >
                    {$t("common.delete")}
                  </button>
                {/if}
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
    message={$t("groups.confirm_delete", { gid: deleteTarget.gid })}
    onConfirm={() => {
      const target = deleteTarget;
      deleteTarget = null;
      if (target) void remove(target);
    }}
    onCancel={() => (deleteTarget = null)}
  />
{/if}
