import { useState } from "react";

type Props = {
  uid: string;
  groups: string[];
  allGroups: string[];
  onSave: (next: string[]) => Promise<boolean>;
  onCreateGroup?: (gid: string) => Promise<{ ok: boolean; error?: string }>;
  onCancel: () => void;
};

export function GroupsEditor({
  uid,
  groups,
  allGroups,
  onSave,
  onCreateGroup,
  onCancel,
}: Props) {
  const [draft, setDraft] = useState<string[]>(groups);
  const [newGid, setNewGid] = useState("");

  const add = (gid: string) => {
    setNewGid("");
    setDraft((d) => (d.includes(gid) ? d : [...d, gid]));
  };

  const remove = (gid: string) => setDraft((d) => d.filter((x) => x !== gid));

  const q = newGid.trim().toLowerCase();
  const suggestions = q
    ? allGroups
        .filter((gid) => !draft.includes(gid) && gid.toLowerCase().includes(q))
        .sort(
          (a, b) =>
            Number(b.toLowerCase().startsWith(q)) -
            Number(a.toLowerCase().startsWith(q)),
        )
    : [];

  const resolveAndAdd = async (raw: string) => {
    const gid = raw.trim();
    if (!gid) return;
    const existing = allGroups.find(
      (g) => g.toLowerCase() === gid.toLowerCase(),
    );
    const inDraft = draft.some((g) => g.toLowerCase() === gid.toLowerCase());
    if (existing && !inDraft) {
      add(existing);
      return;
    }
    if (existing && inDraft) {
      setNewGid("");
      return;
    }
    if (!onCreateGroup) return;
    const res = await onCreateGroup(gid);
    if (res.ok) add(gid);
  };

  const keyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      void resolveAndAdd(suggestions[0] ?? newGid);
    }
  };

  const save = async () => {
    if (await onSave(draft)) {
      onCancel();
    }
  };

  const cancel = () => onCancel();

  return (
    <div
      data-testid={`groups-editor-${uid}`}
      style={{
        display: "inline-flex",
        flexWrap: "wrap",
        gap: 4,
        alignItems: "center",
        border: "1px solid #ccc",
        padding: 4,
        minWidth: 220,
      }}
    >
      {draft.map((gid) => (
        <span
          key={gid}
          data-testid={`groups-chip-${uid}-${gid}`}
          style={{
            border: "1px solid #999",
            borderRadius: 8,
            padding: "1px 5px",
          }}
        >
          {gid}{" "}
          <button
            type="button"
            data-testid={`groups-remove-${uid}-${gid}`}
            onClick={() => remove(gid)}
          >
            ×
          </button>
        </span>
      ))}
      <input
        data-testid={`groups-new-input-${uid}`}
        placeholder="ajouter / créer un groupe"
        value={newGid}
        onChange={(e) => setNewGid(e.target.value)}
        onKeyDown={keyDown}
      />
      <ul
        data-testid={`groups-suggest-list-${uid}`}
        style={{ margin: 0, padding: 0, listStyle: "none" }}
      >
        {suggestions.map((gid) => (
          <li key={gid}>
            <button
              type="button"
              data-testid={`groups-suggest-${uid}-${gid}`}
              onClick={() => add(gid)}
            >
              {gid}
            </button>
          </li>
        ))}
      </ul>
      <div>
        <button
          type="button"
          data-testid={`groups-save-${uid}`}
          onClick={() => void save()}
        >
          Valider
        </button>
        <button
          type="button"
          data-testid={`groups-cancel-${uid}`}
          onClick={cancel}
        >
          Annuler
        </button>
      </div>
    </div>
  );
}
