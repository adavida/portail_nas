import { deleteGroup, updateGroup } from "../api/groups";
import type { Group } from "../api/groups";
import EditableCell from "./EditableCell";
import { toast } from "./Toaster";

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
      <td>{group.gid}</td>
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
      <td>
        <button
          className="btn btn-danger"
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
    <table data-testid="groups-table" className="table">
      <thead>
        <tr>
          <th>GID</th>
          <th>Nom</th>
          <th>Description</th>
          <th>Action</th>
        </tr>
      </thead>
      <tbody>
        {groups.length === 0 ? (
          <tr>
            <td colSpan={4} className="center" data-testid="groups-empty">
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
