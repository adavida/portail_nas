<script lang="ts">
  import { untrack } from "svelte";
  import { deleteGroup, updateGroup, type Group } from "../api/groups";
  import FieldRow from "./FieldRow.svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import { t, translate } from "../i18n";
  import { toast } from "./toast";

  type Props = {
    group: Group;
    canDelete: boolean;  
    onClose: () => void;
    onUpdated?: () => void;
    onDeleted?: () => void;
  };

  let { group, canDelete, onClose, onUpdated, onDeleted }: Props = $props();

  // draft initialized once: the modal is remounted on each opening
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
    toast(translate("common.saved"));
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
    toast(translate("groups.deleted", { gid: group.gid }));
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

<div
  class="modal-backdrop"
  data-testid={`group-modal-${group.gid}`}
  role="dialog"
  aria-modal="true"
  aria-label={$t("modal.group_edit")}
>
  <div class="modal card">
    <div class="modal-header">
      <h3>{group.gid}</h3>
      <button
        class="btn"
        type="button"
        aria-label={$t("nav.close")}
        data-testid={`modal-close-${group.gid}`}
        onclick={onClose}
      >
        ×
      </button>
    </div>

    <FieldRow
      label={$t("field.name")}
      id={`modal-name-${group.gid}`}
      bind:value={name}
      onValidate={() => void saveInfos()}
      onCancel={onClose}
    />
    <FieldRow
      label={$t("field.description")}
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
          {$t("common.delete")}
        </button>
      </div>
    {/if}
  </div>
</div>

{#if confirmDelete}
  <ConfirmModal
    message={$t("groups.confirm_delete", { gid: group.gid })}
    onConfirm={() => void remove()}
    onCancel={() => (confirmDelete = false)}
  />
{/if}
