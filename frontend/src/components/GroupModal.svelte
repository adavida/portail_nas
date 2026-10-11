<script lang="ts">
  import { untrack } from "svelte";
  import { deleteGroup, updateGroup, type Group } from "../api/groups";
  import FieldRow from "./FieldRow.svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import { toast } from "./toast";

  type Props = {
    group: Group;
    canDelete: boolean;  
    onClose: () => void;
    onUpdated?: () => void;
    onDeleted?: () => void;
  };

  let { group, canDelete, onClose, onUpdated, onDeleted }: Props = $props();

  // draft initialisé une fois : le modal est remonté à chaque ouverture
  let name = $state(untrack(() => group.name));
  let description = $state(untrack(() => group.description));
  let confirmDelete = $state(false);

  function onWindowKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && !confirmDelete) onClose();
  }

  async function saveInfos() {
    try {
      await updateGroup(group.gid, { name, description });
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    onUpdated?.();
    toast("Modifications enregistrées");
  }

  async function remove() {
    try {
      await deleteGroup(group.gid);
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    onDeleted?.();
    onClose();
    toast(`Groupe ${group.gid} supprimé`);
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

<div
  class="modal-backdrop"
  data-testid={`group-modal-${group.gid}`}
  role="dialog"
  aria-modal="true"
  aria-label="Édition groupe"
>
  <div class="modal card">
    <div class="modal-header">
      <h3>{group.gid}</h3>
      <button
        class="btn"
        type="button"
        aria-label="Fermer"
        data-testid={`modal-close-${group.gid}`}
        onclick={onClose}
      >
        ×
      </button>
    </div>

    <FieldRow
      label="Nom"
      id={`modal-name-${group.gid}`}
      bind:value={name}
      onValidate={() => void saveInfos()}
      onCancel={onClose}
    />
    <FieldRow
      label="Description"
      id={`modal-description-${group.gid}`}
      bind:value={description}
      onValidate={() => void saveInfos()}
      onCancel={onClose}
    />

    {#if canDelete}
      <div>
        <button
          class="btn btn-danger"
          type="button"
          data-testid={`group-delete-button-${group.gid}`}
          onclick={() => (confirmDelete = true)}
        >
          Supprimer
        </button>
      </div>
    {/if}
  </div>
</div>

{#if confirmDelete}
  <ConfirmModal
    message={`Supprimer le groupe ${group.gid} ?`}
    onConfirm={() => void remove()}
    onCancel={() => (confirmDelete = false)}
  />
{/if}
