export type User = { uid: string; name: string; email: string };

import { useState } from "react";

function EditableCell({
  value,
  field,
  uid,
  onSave,
}: {
  value: string;
  field: "name" | "email";
  uid: string;
  onSave: (field: "name" | "email", value: string) => Promise<boolean>;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);

  const save = async () => {
    const ok = await onSave(field, draft);
    if (ok) setEditing(false);
  };

  if (!editing) {
    return (
      <td
        data-testid={`cell-${field}-${uid}`}
        onDoubleClick={() => setEditing(true)}
        style={{ border: "1px solid #ccc", padding: 8, cursor: "text" }}
      >
        {value}
      </td>
    );
  }

  return (
    <td style={{ border: "1px solid #ccc", padding: 8 }}>
      <input
        data-testid={`edit-input-${field}-${uid}`}
        autoFocus
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={save}
        onKeyDown={(e) => {
          if (e.key === "Enter") save();
          if (e.key === "Escape") setEditing(false);
        }}
      />
    </td>
  );
}

function UserRow({
  user,
  onUpdated,
  onDeleted,
}: {
  user: User;
  onUpdated?: () => void;
  onDeleted?: () => void;
}) {
  const [password, setPassword] = useState("");
  const [msg, setMsg] = useState<string | null>(null);
  const [isError, setIsError] = useState(false);

  const submit = async () => {
    if (!password) {
      setMsg("mot de passe requis");
      setIsError(true);
      return;
    }
    setMsg(null);
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
      setMsg(j.error || `error ${res.status}`);
      setIsError(true);
      return;
    }
    setPassword("");
    setMsg("ok");
    setIsError(false);
  };

  const remove = async () => {
    setMsg(null);
    const res = await fetch(`/api/users/${encodeURIComponent(user.uid)}`, {
      method: "DELETE",
    });
    if (!res.ok) {
      const j = await res.json().catch(() => ({}));
      setMsg(j.error || `error ${res.status}`);
      setIsError(true);
      return;
    }
    onDeleted?.();
  };

  const saveField = async (field: "name" | "email", newValue: string) => {
    setMsg(null);
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
      setMsg(j.error || `error ${res.status}`);
      setIsError(true);
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
        uid={user.uid}
        onSave={saveField}
      />
      <EditableCell
        value={user.email}
        field="email"
        uid={user.uid}
        onSave={saveField}
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
          {msg && (
            <span
              data-testid={
                isError
                  ? `password-error-${user.uid}`
                  : `password-success-${user.uid}`
              }
            >
              {msg}
            </span>
          )}
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
  const [error, setError] = useState<string | null>(null);

  const submit = async () => {
    if (!uid.trim() || !name.trim() || !password) {
      setError("uid, nom et mot de passe requis");
      return;
    }
    setError(null);
    const res = await fetch("/api/users", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ uid, name, email, password }),
    });
    if (!res.ok) {
      const j = await res.json().catch(() => ({}));
      setError(j.error || `error ${res.status}`);
      return;
    }
    setUid("");
    setName("");
    setEmail("");
    setPassword("");
    onCreated?.();
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
        {error && <span data-testid="create-error"> {error}</span>}
      </td>
    </tr>
  );
}

export function UsersTable({
  users,
  onCreated,
  onUpdated,
  onDeleted,
}: {
  users: User[];
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
              colSpan={5}
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
              onUpdated={onUpdated}
              onDeleted={onDeleted}
            />
          ))
        )}
      </tbody>
    </table>
  );
}
