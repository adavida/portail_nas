import { useState } from "react";
import "./GroupsEditor.scss";

type Props = {
  uid: string;
  groups: string[];
  allGroups: string[];
  onSave: (next: string[]) => Promise<boolean>;
  onCreateGroup?: (gid: string) => Promise<{ ok: boolean; error?: string }>;
  onCancel: () => void;
};

const MASTER_GID = "user";

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

  const remove = (gid: string) => {
    if (gid === MASTER_GID) return;
    setDraft((d) => d.filter((x) => x !== gid));
  };

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
    } else if (e.key === "Backspace" && !newGid) {
      const last = draft.filter((g) => g !== MASTER_GID).pop();
      if (last) remove(last);
    }
  };

  const save = async () => {
    if (await onSave(draft)) {
      onCancel();
    }
  };

  const cancel = () => onCancel();

  return (
    <div data-testid={`groups-editor-${uid}`} className="groups-editor">
      <div className="field">
        {draft.map((gid) => (
          <span
            key={gid}
            data-testid={`groups-chip-${uid}-${gid}`}
            className="chip"
          >
            {gid}
            {gid !== MASTER_GID && (
              <button
                type="button"
                aria-label={`retirer ${gid}`}
                data-testid={`groups-remove-${uid}-${gid}`}
                onClick={() => remove(gid)}
              >
                ×
              </button>
            )}
          </span>
        ))}
        <input
          autoFocus
          className="input"
          data-testid={`groups-new-input-${uid}`}
          placeholder={draft.length ? "" : "ajouter / créer un groupe"}
          value={newGid}
          onChange={(e) => setNewGid(e.target.value)}
          onKeyDown={keyDown}
        />
      </div>
      {suggestions.length > 0 && (
        <ul data-testid={`groups-suggest-list-${uid}`} className="suggest">
          {suggestions.map((gid) => (
            <li key={gid}>
              <button
                className="btn"
                type="button"
                data-testid={`groups-suggest-${uid}-${gid}`}
                onClick={() => add(gid)}
              >
                {gid}
              </button>
            </li>
          ))}
        </ul>
      )}
      <div className="row-actions">
        <button
          className="btn btn-primary"
          type="button"
          data-testid={`groups-save-${uid}`}
          onClick={() => void save()}
        >
          Valider
        </button>
        <button
          className="btn"
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
