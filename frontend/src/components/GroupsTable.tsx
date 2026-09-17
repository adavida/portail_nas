export type Group = {
  gid: string;
  name: string;
  description: string;
  members: string[];
};

import { useState } from "react";
import EditableCell from "./EditableCell";

const td = { border: "1px solid #ccc", padding: 8 };

function GroupRow({
  group,
  onUpdated,
  onDeleted,
}: {
  group: Group;
  onUpdated?: () => void;
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

  const saveField = async (field: string, newValue: string) => {
    setMsg(null);
    const res = await fetch(`/api/groups/${encodeURIComponent(group.gid)}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(
        field === "name"
          ? { name: newValue, description: group.description }
          : { name: group.name, description: newValue },
      ),
    });
    if (!res.ok) {
      const j = await res.json().catch(() => ({}));
      setMsg(j.error || `error ${res.status}`);
      return false;
    }
    onUpdated?.();
    return true;
  };

  return (
    <tr data-testid={`group-row-${group.gid}`}>
      <td style={td}>{group.gid}</td>
      <EditableCell
        value={group.name}
        field="name"
        rowId={group.gid}
        onSave={saveField}
      />
      <EditableCell
        value={group.description}
        field="description"
        rowId={group.gid}
        onSave={saveField}
      />
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

export function GroupsTable({
  groups,
  onUpdated,
  onDeleted,
}: {
  groups: Group[];
  onUpdated?: () => void;
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
            <GroupRow
              key={g.gid}
              group={g}
              onUpdated={onUpdated}
              onDeleted={onDeleted}
            />
          ))
        )}
      </tbody>
    </table>
  );
}
