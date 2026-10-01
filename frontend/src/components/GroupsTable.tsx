import { deleteGroup, updateGroup } from "../api/groups";
import type { Group } from "../api/groups";
import EditableCell from "./EditableCell";
import { toast } from "./Toaster";

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
  const remove = async () => {
    try {
      await deleteGroup(group.gid);
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    onDeleted?.();
    toast(`Groupe ${group.gid} supprimé`);
  };

  const saveField = async (field: string, newValue: string) => {
    const body =
      field === "name"
        ? { name: newValue, description: group.description }
        : { name: group.name, description: newValue };
    try {
      await updateGroup(group.gid, body);
    } catch (e) {
      toast((e as Error).message, true);
      return false;
    }
    toast("Modifications enregistrées");
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
