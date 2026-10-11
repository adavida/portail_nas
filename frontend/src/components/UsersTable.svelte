<script lang="ts">
  import {
    createUser,
    deleteUser,
    updatePassword,
    updateUser,
    type User,
  } from "../api/users";
  import { addMember, createGroup, removeMember } from "../api/groups";
  import EditableCell from "./EditableCell.svelte";
  import GroupsEditor from "./GroupsEditor.svelte";
  import { toast } from "./toast";

  type Props = {
    users: User[];
    allGroups?: string[];
    onCreated?: () => void;
    onUpdated?: () => void;
    onDeleted?: () => void;
  };

  let { users, allGroups = [], onCreated, onUpdated, onDeleted }: Props =
    $props();

  let editingGroupsUid = $state<string | null>(null);
  let passwords = $state<Record<string, string>>({});

  let newUid = $state("");
  let newName = $state("");
  let newEmail = $state("");
  let newPassword = $state("");

  async function saveGroups(
    user: User,
    next: string[],
  ): Promise<boolean> {
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
    toast(`Groupe ${gid} créé`);
    onUpdated?.();
    return { ok: true };
  }

  async function submitPassword(user: User) {
    const password = passwords[user.uid] ?? "";
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
    passwords[user.uid] = "";
    toast("Mot de passe modifié");
  }

  async function removeUser(user: User) {
    try {
      await deleteUser(user.uid);
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    onDeleted?.();
    toast(`Utilisateur ${user.uid} supprimé`);
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

  async function submitCreate() {
    if (!newUid.trim() || !newName.trim() || !newPassword) {
      toast("uid, nom et mot de passe requis", true);
      return;
    }
    const uid = newUid;
    try {
      await createUser({
        uid: newUid,
        name: newName,
        email: newEmail,
        password: newPassword,
      });
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    newUid = "";
    newName = "";
    newEmail = "";
    newPassword = "";
    onCreated?.();
    toast(`Utilisateur ${uid} créé`);
  }
</script>

<table data-testid="users-table" class="table">
  <thead>
    <tr>
      <th>UID</th>
      <th>Nom</th>
      <th>Email</th>
      <th>Groupes</th>
      <th>Mot de passe</th>
      <th>Action</th>
    </tr>
  </thead>
  <tbody>
    {#if onCreated}
      <tr data-testid="create-row">
        <td>
          <input
            class="input"
            data-testid="input-uid"
            placeholder="uid"
            bind:value={newUid}
          />
        </td>
        <td>
          <input
            class="input"
            data-testid="input-name"
            placeholder="nom"
            bind:value={newName}
          />
        </td>
        <td>
          <input
            class="input"
            data-testid="input-email"
            placeholder="email"
            bind:value={newEmail}
          />
        </td>
        <td></td>
        <td>
          <input
            class="input"
            data-testid="input-password"
            placeholder="mot de passe"
            type="password"
            bind:value={newPassword}
          />
        </td>
        <td>
          <button
            class="btn btn-primary"
            data-testid="create-button"
            onclick={() => void submitCreate()}
            type="button"
          >
            Créer
          </button>
        </td>
      </tr>
    {/if}
    {#if users.length === 0}
      <tr>
        <td colSpan={6} class="center" data-testid="users-empty">
          Aucun utilisateur
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
              ondblclick={() => (editingGroupsUid = u.uid)}
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
              />
            </td>
          {/if}
          <td>
            <span class="row-actions">
              <input
                class="input"
                data-testid={`password-input-${u.uid}`}
                placeholder="nouveau mdp"
                type="password"
                bind:value={passwords[u.uid]}
              />
              <button
                class="btn btn-primary"
                data-testid={`password-button-${u.uid}`}
                onclick={() => void submitPassword(u)}
                type="button"
              >
                Modifier
              </button>
            </span>
          </td>
          <td>
            <button
              class="btn btn-danger"
              data-testid={`delete-button-${u.uid}`}
              onclick={() => void removeUser(u)}
              type="button"
            >
              Supprimer
            </button>
          </td>
        </tr>
      {/each}
    {/if}
  </tbody>
</table>
