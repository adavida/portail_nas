<script lang="ts">
  import { untrack } from "svelte";
  import {
    deleteUser,
    updatePassword,
    updateUser,
    type User,
  } from "../api/users";
  import GroupsEditor from "./GroupsEditor.svelte";
  import FieldRow from "./FieldRow.svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import { t, translate } from "../i18n";
  import { toast } from "./toast";
  import { createGroupWithMember, saveGroups } from "./user_groups";

  type Props = {
    user: User;
    allGroups?: string[];
    onClose: () => void;
    onUpdated?: () => void;
    onDeleted?: () => void;
  };

  let { user, allGroups = [], onClose, onUpdated, onDeleted }: Props = $props();

  // draft initialized once: the modal is remounted on each opening
  let name = $state(untrack(() => user.name));
  let email = $state(untrack(() => user.email));
  let password = $state("");
  let confirmDelete = $state(false);

  function onWindowKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && !confirmDelete) onClose();
  }

  async function saveInfos() {
    try {
      await updateUser(user.uid, { name, email });
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    onUpdated?.();
    toast(translate("common.saved"));
  }

  async function submitPassword() {
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
    password = "";
    toast(translate("common.password_changed"));
  }

  async function removeUser() {
    try {
      await deleteUser(user.uid);
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    onDeleted?.();
    onClose();
    toast(translate("users.deleted", { uid: user.uid }));
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

<div
  class="modal-backdrop"
  data-testid={`user-modal-${user.uid}`}
  role="dialog"
  aria-modal="true"
  aria-label={$t("modal.user_edit")}
>
  <div class="modal card">
    <div class="modal-header">
      <h3>{user.uid}</h3>
      <button
        class="btn"
        type="button"
        aria-label={$t("nav.close")}
        data-testid={`modal-close-${user.uid}`}
        onclick={onClose}
      >
        ×
      </button>
    </div>

    <FieldRow
      label={$t("field.name")}
      id={`modal-name-${user.uid}`}
      bind:value={name}
      onValidate={() => void saveInfos()}
      onCancel={onClose}
    />
    <FieldRow
      label={$t("field.email")}
      id={`modal-email-${user.uid}`}
      bind:value={email}
      onValidate={() => void saveInfos()}
      onCancel={onClose}
    />

    <h4>{$t("field.password")}</h4>
    <div class="row-actions">
      <input
        class="input"
        type="password"
        placeholder={$t("common.password_new")}
        data-testid={`password-input-${user.uid}`}
        bind:value={password}
      />
      <button
        class="btn btn-primary"
        type="button"
        data-testid={`password-button-${user.uid}`}
        onclick={() => void submitPassword()}
      >
        {$t("common.edit")}
      </button>
    </div>

    <h4>{$t("nav.groups")}</h4>
    <GroupsEditor
      uid={user.uid}
      groups={user.groups ?? []}
      {allGroups}
      onSave={(next) => saveGroups(user.uid, user.groups ?? [], next, onUpdated)}
      onCreateGroup={(gid) => createGroupWithMember(user.uid, gid, onUpdated)}
      onCancel={onClose}
    />

    <div>
      <button
        class="btn btn-danger"
        type="button"
        data-testid={`delete-button-${user.uid}`}
        onclick={() => (confirmDelete = true)}
      >
        {$t("common.delete")}
      </button>
    </div>
  </div>
</div>

{#if confirmDelete}
  <ConfirmModal
    message={$t("users.confirm_delete", { uid: user.uid })}
    onConfirm={() => void removeUser()}
    onCancel={() => (confirmDelete = false)}
  />
{/if}
