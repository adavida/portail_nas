<script lang="ts">
  import { deleteGroup, updateGroup, type Group } from "../api/groups";
  import EditableCell from "./EditableCell.svelte";
  import { toast } from "./toast";

  type Props = {
    groups: Group[];
    onUpdated?: () => void;
    onDeleted?: () => void;
  };

  const PROTECTED_GIDS = ["admin", "user"];
  const isProtected = (gid: string) => PROTECTED_GIDS.includes(gid);
  const isHidden = (gid: string) => gid === "user";

  let { groups, onUpdated, onDeleted }: Props = $props();

  const visible = $derived(groups.filter((g) => !isHidden(g.gid)));

  async function remove(group: Group) {
    try {
      await deleteGroup(group.gid);
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    onDeleted?.();
    toast(`Groupe ${group.gid} supprimé`);
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
    toast("Modifications enregistrées");
    onUpdated?.();
    return true;
  }
</script>

<table data-testid="groups-table" class="table">
  <thead>
    <tr>
      <th>GID</th>
      <th>Nom</th>
      <th>Description</th>
      <th>Action</th>
    </tr>
  </thead>
  <tbody>
    {#if visible.length === 0}
      <tr>
        <td colSpan={4} class="center" data-testid="groups-empty">
          Aucun groupe
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
                onclick={() => void remove(g)}
                type="button"
              >
                Supprimer
              </button>
            {/if}
          </td>
        </tr>
      {/each}
    {/if}
  </tbody>
</table>
