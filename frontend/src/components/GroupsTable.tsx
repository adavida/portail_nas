export type Group = { gid: string; name: string; description: string };

import { useState } from "react";

const td = { border: "1px solid #ccc", padding: 8 };

function GroupRow({
  group,
  onDeleted,
}: {
  group: Group;
  onDeleted?: () => void;
}) {
  const [msg, setMsg] = useState<string | null>(null);

  const remove = async () => {
    setMsg(null);
    const res = await fetch(`/api/groups/${encodeURIComponent(group.gid)}`, {
      method: "DELETE",
    });
    if (!res.ok) {
      const j = await res.json().catch(() => ({}));
      setMsg(j.error || `error ${res.status}`);
      return;
    }
    onDeleted?.();
  };

  return (
    <tr data-testid={`group-row-${group.gid}`}>
      <td style={td}>{group.gid}</td>
      <td style={td}>{group.name}</td>
      <td style={td}>{group.description}</td>
      <td style={td}>
        <button
          data-testid={`group-delete-button-${group.gid}`}
          onClick={remove}
          type="button"
        >
          Supprimer
        </button>
        {msg && <span data-testid={`group-error-${group.gid}`}> {msg}</span>}
      </td>
    </tr>
  );
}

function CreateRow({ onCreated }: { onCreated?: () => void }) {
  const [gid, setGid] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [error, setError] = useState<string | null>(null);

  const submit = async () => {
    if (!gid.trim() || !name.trim() || !description.trim()) {
      setError("gid, nom et description requis");
      return;
    }
    setError(null);
    const res = await fetch("/api/groups", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ gid, name, description }),
    });
    if (!res.ok) {
      const j = await res.json().catch(() => ({}));
      setError(j.error || `error ${res.status}`);
      return;
    }
    setGid("");
    setName("");
    setDescription("");
    onCreated?.();
  };

  return (
    <tr data-testid="group-create-row">
      <td style={td}>
        <input
          data-testid="input-gid"
          placeholder="gid"
          value={gid}
          onChange={(e) => setGid(e.target.value)}
        />
      </td>
      <td style={td}>
        <input
          data-testid="input-name"
          placeholder="nom"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </td>
      <td style={td}>
        <input
          data-testid="input-description"
          placeholder="description"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
        />
      </td>
      <td style={td}>
        <button
          data-testid="group-create-button"
          onClick={submit}
          type="button"
        >
          Créer
        </button>
        {error && <span data-testid="group-create-error"> {error}</span>}
      </td>
    </tr>
  );
}

export function GroupsTable({
  groups,
  onCreated,
  onDeleted,
}: {
  groups: Group[];
  onCreated?: () => void;
  onDeleted?: () => void;
}) {
  return (
    <table
      data-testid="groups-table"
      style={{ borderCollapse: "collapse", width: "100%" }}
    >
      <thead>
        <tr>
          <th style={{ ...td, textAlign: "left" }}>GID</th>
          <th style={{ ...td, textAlign: "left" }}>Nom</th>
          <th style={{ ...td, textAlign: "left" }}>Description</th>
          <th style={{ ...td, textAlign: "left" }}>Action</th>
        </tr>
      </thead>
      <tbody>
        {onCreated && <CreateRow onCreated={onCreated} />}
        {groups.length === 0 ? (
          <tr>
            <td
              colSpan={4}
              style={{ ...td, textAlign: "center" }}
              data-testid="groups-empty"
            >
              Aucun groupe
            </td>
          </tr>
        ) : (
          groups.map((g) => (
            <GroupRow key={g.gid} group={g} onDeleted={onDeleted} />
          ))
        )}
      </tbody>
    </table>
  );
}
