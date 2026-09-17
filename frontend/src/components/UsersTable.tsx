export type User = {
  uid: string;
  name: string;
  email: string;
  groups?: string[];
};

import { useState } from "react";
import EditableCell from "./EditableCell";
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
  const [draft, setDraft] = useState<string[]>(groups);
  const [newGid, setNewGid] = useState("");

  const toggle = (gid: string) =>
    setDraft((d) =>
      d.includes(gid) ? d.filter((x) => x !== gid) : [...d, gid],
    );

  const save = async () => {
    const res = await onSave(draft);
    if (res.ok) {
      setEditing(false);
    }
  };

  const cancel = () => {
    setDraft(groups);
    setEditing(false);
  };

  const createGroup = async () => {
    const gid = newGid.trim();
    if (!gid || !onCreateGroup) return;
    const res = await onCreateGroup(gid);
    if (res.ok) {
      setDraft((d) => [...d, gid]);
      setNewGid("");
    }
  };

  if (!editing) {
    return (
      <td
        data-testid={`cell-groups-${uid}`}
        onDoubleClick={() => setEditing(true)}
        style={{ border: "1px solid #ccc", padding: 8, cursor: "text" }}
      >
        {groups.length ? groups.join(" ") : "-"}
      </td>
    );
  }

  return (
    <td style={{ border: "1px solid #ccc", padding: 8 }}>
      {allGroups.map((gid) => (
        <button
          key={gid}
          type="button"
          data-testid={`groups-toggle-${uid}-${gid}`}
          style={{
            margin: 2,
            fontWeight: draft.includes(gid) ? "bold" : "normal",
            textDecoration: draft.includes(gid) ? "underline" : "none",
          }}
          onClick={() => toggle(gid)}
        >
          {draft.includes(gid) ? "✓" : "✗"} {gid}
        </button>
      ))}
      <input
        data-testid={`groups-new-input-${uid}`}
        placeholder="nouveau groupe (gid)"
        value={newGid}
        onChange={(e) => setNewGid(e.target.value)}
      />
      {onCreateGroup && (
        <button
          type="button"
          data-testid={`groups-new-button-${uid}`}
          onClick={createGroup}
        >
          Créer
        </button>
      )}
      <button type="button" data-testid={`groups-save-${uid}`} onClick={save}>
        Valider
      </button>
      <button
        type="button"
        data-testid={`groups-cancel-${uid}`}
        onClick={cancel}
      >
        Annuler
      </button>
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

    for (const gid of adds) {
      const res = await fetch(
        `/api/groups/${encodeURIComponent(gid)}/members`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ uid: user.uid }),
        },
      );
      if (!res.ok) {
        const j = await res.json().catch(() => ({}));
        const error = j.error || `error ${res.status}`;
        toast(error, true);
        return { ok: false, error };
      }
    }

    for (const gid of removes) {
      const res = await fetch(
        `/api/groups/${encodeURIComponent(gid)}/members/${encodeURIComponent(user.uid)}`,
        { method: "DELETE" },
      );
      if (!res.ok) {
        const j = await res.json().catch(() => ({}));
        const error = j.error || `error ${res.status}`;
        toast(error, true);
        return { ok: false, error };
      }
    }

    toast("Modifications enregistrées");
    onUpdated?.();
    return { ok: true };
  };

  const createGroupWithMember = async (
    gid: string,
  ): Promise<{ ok: boolean; error?: string }> => {
    const res = await fetch("/api/groups", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        gid,
        name: gid,
        description: "",
        members: [user.uid],
      }),
    });
    if (!res.ok) {
      const j = await res.json().catch(() => ({}));
      const error = j.error || `error ${res.status}`;
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
    const res = await fetch(
      `/api/users/${encodeURIComponent(user.uid)}/password`,
      {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ password }),
      },
    );
    if (!res.ok) {
      const j = await res.json().catch(() => ({}));
      toast(j.error || `error ${res.status}`, true);
      return;
    }
    setPassword("");
    toast("Mot de passe modifié");
  };

  const remove = async () => {
    const res = await fetch(`/api/users/${encodeURIComponent(user.uid)}`, {
      method: "DELETE",
    });
    if (!res.ok) {
      const j = await res.json().catch(() => ({}));
      toast(j.error || `error ${res.status}`, true);
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
    const res = await fetch(`/api/users/${encodeURIComponent(user.uid)}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    if (!res.ok) {
      const j = await res.json().catch(() => ({}));
      toast(j.error || `error ${res.status}`, true);
      return false;
    }
    onUpdated?.();
    return true;
  };

  return (
    <tr data-testid={`user-row-${user.uid}`}>
      <td style={{ border: "1px solid #ccc", padding: 8 }}>{user.uid}</td>
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
      <td style={{ border: "1px solid #ccc", padding: 8 }}>
        <span style={{ display: "flex", gap: 8, alignItems: "center" }}>
          <input
            data-testid={`password-input-${user.uid}`}
            placeholder="nouveau mdp"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
          <button
            data-testid={`password-button-${user.uid}`}
            onClick={submit}
            type="button"
          >
            Modifier
          </button>
        </span>
      </td>
      <td style={{ border: "1px solid #ccc", padding: 8 }}>
        <button
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
    const res = await fetch("/api/users", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ uid, name, email, password }),
    });
    if (!res.ok) {
      const j = await res.json().catch(() => ({}));
      toast(j.error || `error ${res.status}`, true);
      return;
    }
    setUid("");
    setName("");
    setEmail("");
    setPassword("");
    onCreated?.();
    toast(`Utilisateur ${uid} créé${gid ? ` + groupe ${gid}` : ""}`);
  };

  return (
    <tr data-testid="create-row">
      <td style={{ border: "1px solid #ccc", padding: 8 }}>
        <input
          data-testid="input-uid"
          placeholder="uid"
          value={uid}
          onChange={(e) => setUid(e.target.value)}
        />
      </td>
      <td style={{ border: "1px solid #ccc", padding: 8 }}>
        <input
          data-testid="input-name"
          placeholder="nom"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </td>
      <td style={{ border: "1px solid #ccc", padding: 8 }}>
        <input
          data-testid="input-email"
          placeholder="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
        />
      </td>
      <td style={{ border: "1px solid #ccc", padding: 8 }} />
      <td style={{ border: "1px solid #ccc", padding: 8 }}>
        <input
          data-testid="input-password"
          placeholder="mot de passe"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
      </td>
      <td style={{ border: "1px solid #ccc", padding: 8 }}>
        <button data-testid="create-button" onClick={submit} type="button">
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
    <table
      data-testid="users-table"
      style={{ borderCollapse: "collapse", width: "100%" }}
    >
      <thead>
        <tr>
          <th
            style={{ border: "1px solid #ccc", padding: 8, textAlign: "left" }}
          >
            UID
          </th>
          <th
            style={{ border: "1px solid #ccc", padding: 8, textAlign: "left" }}
          >
            Nom
          </th>
          <th
            style={{ border: "1px solid #ccc", padding: 8, textAlign: "left" }}
          >
            Email
          </th>
          <th
            style={{ border: "1px solid #ccc", padding: 8, textAlign: "left" }}
          >
            Groupes
          </th>
          <th
            style={{ border: "1px solid #ccc", padding: 8, textAlign: "left" }}
          >
            Mot de passe
          </th>
          <th
            style={{ border: "1px solid #ccc", padding: 8, textAlign: "left" }}
          >
            Action
          </th>
        </tr>
      </thead>
      <tbody>
        {onCreated && <CreateRow onCreated={onCreated} />}
        {users.length === 0 ? (
          <tr>
            <td
              colSpan={6}
              style={{
                border: "1px solid #ccc",
                padding: 8,
                textAlign: "center",
              }}
              data-testid="users-empty"
            >
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
