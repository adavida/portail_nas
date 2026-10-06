import { useState } from "react";
import {
  createUser,
  deleteUser,
  updatePassword,
  updateUser,
} from "../api/users";
import type { User } from "../api/users";
import { addMember, createGroup, removeMember } from "../api/groups";
import EditableCell from "./EditableCell";
import { GroupsEditor } from "./GroupsEditor";
import { toast } from "./Toaster";

function GroupsCell({
  uid,
  groups,
  allGroups,
  onSave,
  onCreateGroup,
}: {
  uid: string;
  groups: string[];
  allGroups: string[];
  onSave: (next: string[]) => Promise<{ ok: boolean; error?: string }>;
  onCreateGroup?: (gid: string) => Promise<{ ok: boolean; error?: string }>;
}) {
  const [editing, setEditing] = useState(false);

  if (!editing) {
    return (
      <td
        data-testid={`cell-groups-${uid}`}
        className="editable"
        onDoubleClick={() => setEditing(true)}
      >
        {groups.length ? groups.join(" ") : "-"}
      </td>
    );
  }

  return (
    <td>
      <GroupsEditor
        uid={uid}
        groups={groups}
        allGroups={allGroups}
        onSave={async (next) => (await onSave(next)).ok}
        onCreateGroup={onCreateGroup}
        onCancel={() => setEditing(false)}
      />
    </td>
  );
}

function UserRow({
  user,
  allGroups,
  onUpdated,
  onDeleted,
}: {
  user: User;
  allGroups?: string[];
  onUpdated?: () => void;
  onDeleted?: () => void;
}) {
  const [password, setPassword] = useState("");
  const groups = user.groups ?? [];

  const saveGroups = async (
    next: string[],
  ): Promise<{ ok: boolean; error?: string }> => {
    const adds = next.filter((gid) => !groups.includes(gid));
    const removes = groups.filter((gid) => !next.includes(gid));

    try {
      for (const gid of adds) await addMember(gid, user.uid);
      for (const gid of removes) await removeMember(gid, user.uid);
    } catch (e) {
      const error = (e as Error).message;
      toast(error, true);
      return { ok: false, error };
    }

    toast("Modifications enregistrées");
    onUpdated?.();
    return { ok: true };
  };

  const createGroupWithMember = async (
    gid: string,
  ): Promise<{ ok: boolean; error?: string }> => {
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
  };

  const submit = async () => {
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
    setPassword("");
    toast("Mot de passe modifié");
  };

  const remove = async () => {
    try {
      await deleteUser(user.uid);
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    onDeleted?.();
    toast(`Utilisateur ${user.uid} supprimé`);
  };

  const saveField = async (field: string, newValue: string) => {
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
  };

  return (
    <tr data-testid={`user-row-${user.uid}`}>
      <td>{user.uid}</td>
      <EditableCell
        value={user.name}
        field="name"
        rowId={user.uid}
        onSave={saveField}
      />
      <EditableCell
        value={user.email}
        field="email"
        rowId={user.uid}
        onSave={saveField}
      />
      <GroupsCell
        uid={user.uid}
        groups={groups}
        allGroups={allGroups ?? []}
        onSave={saveGroups}
        onCreateGroup={createGroupWithMember}
      />
      <td>
        <span className="row-actions">
          <input
            className="input"
            data-testid={`password-input-${user.uid}`}
            placeholder="nouveau mdp"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
          <button
            className="btn btn-primary"
            data-testid={`password-button-${user.uid}`}
            onClick={submit}
            type="button"
          >
            Modifier
          </button>
        </span>
      </td>
      <td>
        <button
          className="btn btn-danger"
          data-testid={`delete-button-${user.uid}`}
          onClick={remove}
          type="button"
        >
          Supprimer
        </button>
      </td>
    </tr>
  );
}

function CreateRow({ onCreated }: { onCreated?: () => void }) {
  const [uid, setUid] = useState("");
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");

  const submit = async () => {
    if (!uid.trim() || !name.trim() || !password) {
      toast("uid, nom et mot de passe requis", true);
      return;
    }
    try {
      await createUser({ uid, name, email, password });
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    setUid("");
    setName("");
    setEmail("");
    setPassword("");
    onCreated?.();
    toast(`Utilisateur ${uid} créé`);
  };

  return (
    <tr data-testid="create-row">
      <td>
        <input
          className="input"
          data-testid="input-uid"
          placeholder="uid"
          value={uid}
          onChange={(e) => setUid(e.target.value)}
        />
      </td>
      <td>
        <input
          className="input"
          data-testid="input-name"
          placeholder="nom"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </td>
      <td>
        <input
          className="input"
          data-testid="input-email"
          placeholder="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
        />
      </td>
      <td />
      <td>
        <input
          className="input"
          data-testid="input-password"
          placeholder="mot de passe"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
      </td>
      <td>
        <button
          className="btn btn-primary"
          data-testid="create-button"
          onClick={submit}
          type="button"
        >
          Créer
        </button>
      </td>
    </tr>
  );
}

export function UsersTable({
  users,
  allGroups,
  onCreated,
  onUpdated,
  onDeleted,
}: {
  users: User[];
  allGroups?: string[];
  onCreated?: () => void;
  onUpdated?: () => void;
  onDeleted?: () => void;
}) {
  return (
    <table data-testid="users-table" className="table">
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
        {onCreated && <CreateRow onCreated={onCreated} />}
        {users.length === 0 ? (
          <tr>
            <td colSpan={6} className="center" data-testid="users-empty">
              Aucun utilisateur
            </td>
          </tr>
        ) : (
          users.map((u) => (
            <UserRow
              key={u.uid}
              user={u}
              allGroups={allGroups}
              onUpdated={onUpdated}
              onDeleted={onDeleted}
            />
          ))
        )}
      </tbody>
    </table>
  );
}
