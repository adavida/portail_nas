import { useState } from "react";

function EditableCell({
  value,
  field,
  rowId,
  onSave,
}: {
  value: string;
  field: "name" | "email" | "description";
  rowId: string;
  onSave: (field: string, value: string) => Promise<boolean>;
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
        data-testid={`cell-${field}-${rowId}`}
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
        data-testid={`edit-input-${field}-${rowId}`}
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

export default EditableCell;
