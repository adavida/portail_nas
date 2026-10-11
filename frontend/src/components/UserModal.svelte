<script lang="ts">
  import { untrack } from "svelte";
  import {
    deleteUser,
    updatePassword,
    updateUser,
    type User,
  } from "../api/users";
  import { addMember, createGroup, removeMember } from "../api/groups";
  import GroupsEditor from "./GroupsEditor.svelte";
  import FieldRow from "./FieldRow.svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import { toast } from "./toast";

  type Props = {
    user: User;
    allGroups?: string[];
    onClose: () => void;
    onUpdated?: () => void;
    onDeleted?: () => void;
  };

  let { user, allGroups = [], onClose, onUpdated, onDeleted }: Props = $props();

  // draft initialisé une fois : le modal est remonté à chaque ouverture
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
    toast("Modifications enregistrées");
  }

  async function submitPassword() {
    if (!password) {
      toast("mot de passe requis", true);
      return;
    }
    try {
      await updatePassword(user.uid, password);
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    password = "";
    toast("Mot de passe modifié");
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
    toast(`Utilisateur ${user.uid} supprimé`);
  }

  async function saveGroups(next: string[]): Promise<boolean> {
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
    toast("Modifications enregistrées");
    onUpdated?.();
    return true;
  }

  async function createGroupWithMember(
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
    toast(`Groupe ${gid} créé`);
    onUpdated?.();
    return { ok: true };
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

<div
  class="modal-backdrop"
  data-testid={`user-modal-${user.uid}`}
  role="dialog"
  aria-modal="true"
  aria-label="Édition utilisateur"
>
  <div class="modal card">
    <div class="modal-header">
      <h3>{user.uid}</h3>
      <button
        class="btn"
        type="button"
        aria-label="Fermer"
        data-testid={`modal-close-${user.uid}`}
        onclick={onClose}
      >
        ×
      </button>
    </div>

    <FieldRow
      label="Nom"
      id={`modal-name-${user.uid}`}
      bind:value={name}
      onValidate={() => void saveInfos()}
      onCancel={onClose}
    />
    <FieldRow
      label="Email"
      id={`modal-email-${user.uid}`}
      bind:value={email}
      onValidate={() => void saveInfos()}
      onCancel={onClose}
    />

    <h4>Mot de passe</h4>
    <div class="row-actions">
      <input
        class="input"
        type="password"
        placeholder="nouveau mdp"
        data-testid={`password-input-${user.uid}`}
        bind:value={password}
      />
      <button
        class="btn btn-primary"
        type="button"
        data-testid={`password-button-${user.uid}`}
        onclick={() => void submitPassword()}
      >
        Modifier
      </button>
    </div>

    <h4>Groupes</h4>
    <GroupsEditor
      uid={user.uid}
      groups={user.groups ?? []}
      {allGroups}
      onSave={(next) => saveGroups(next)}
      onCreateGroup={(gid) => createGroupWithMember(gid)}
      onCancel={onClose}
    />

    <div>
      <button
        class="btn btn-danger"
        type="button"
        data-testid={`delete-button-${user.uid}`}
        onclick={() => (confirmDelete = true)}
      >
        Supprimer
      </button>
    </div>
  </div>
</div>

{#if confirmDelete}
  <ConfirmModal
    message={`Supprimer l'utilisateur ${user.uid} ?`}
    onConfirm={() => void removeUser()}
    onCancel={() => (confirmDelete = false)}
  />
{/if}
