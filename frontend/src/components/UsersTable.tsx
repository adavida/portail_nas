export type User = { uid: string; name: string; email: string };

import { useState } from "react";

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
}: {
  users: User[];
  onCreated?: () => void;
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
            <tr key={u.uid} data-testid={`user-row-${u.uid}`}>
              <td style={{ border: "1px solid #ccc", padding: 8 }}>{u.uid}</td>
              <td style={{ border: "1px solid #ccc", padding: 8 }}>{u.name}</td>
              <td style={{ border: "1px solid #ccc", padding: 8 }}>
                {u.email}
              </td>
              <td style={{ border: "1px solid #ccc", padding: 8 }}></td>
              <td style={{ border: "1px solid #ccc", padding: 8 }}></td>
            </tr>
          ))
        )}
      </tbody>
    </table>
  );
}
