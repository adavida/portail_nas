<script lang="ts">
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

  let { uid, groups, allGroups, onSave, onCreateGroup, onCancel }: Props =
    $props();

  let edited = $state<string[] | null>(null);
  const draft = $derived(edited ?? groups);
  let newGid = $state("");

  const q = $derived(newGid.trim().toLowerCase());
  const suggestions = $derived(
    q
      ? allGroups
          .filter((gid) => !draft.includes(gid) && gid.toLowerCase().includes(q))
          .sort(
            (a, b) =>
              Number(b.toLowerCase().startsWith(q)) -
              Number(a.toLowerCase().startsWith(q)),
          )
      : [],
  );
  const visible = $derived(draft.filter((g) => g !== MASTER_GID));

  function add(gid: string) {
    newGid = "";
    if (!draft.includes(gid)) edited = [...draft, gid];
  }

  function remove(gid: string) {
    edited = draft.filter((x) => x !== gid);
  }

  async function resolveAndAdd(raw: string) {
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
      newGid = "";
      return;
    }
    if (!onCreateGroup) return;
    const res = await onCreateGroup(gid);
    if (res.ok) add(gid);
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      void resolveAndAdd(suggestions[0] ?? newGid);
    } else if (e.key === "Backspace" && !newGid) {
      const last = draft.filter((g) => g !== MASTER_GID).pop();
      if (last) remove(last);
    }
  }

  async function save() {
    if (await onSave(draft)) onCancel();
  }
</script>

<div data-testid={`groups-editor-${uid}`} class="groups-editor">
  <div class="field">
    {#each visible as gid (gid)}
      <span data-testid={`groups-chip-${uid}-${gid}`} class="chip">
        {gid}
        <button
          type="button"
          aria-label={`retirer ${gid}`}
          data-testid={`groups-remove-${uid}-${gid}`}
          onclick={() => remove(gid)}
        >
          ×
        </button>
      </span>
    {/each}
    <!-- svelte-ignore a11y_autofocus -->
    <input
      autofocus
      class="input"
      data-testid={`groups-new-input-${uid}`}
      placeholder={visible.length ? "" : "ajouter / créer un groupe"}
      bind:value={newGid}
      onkeydown={onKeyDown}
    />
  </div>
  {#if suggestions.length > 0}
    <ul data-testid={`groups-suggest-list-${uid}`} class="suggest">
      {#each suggestions as gid (gid)}
        <li>
          <button
            class="btn"
            type="button"
            data-testid={`groups-suggest-${uid}-${gid}`}
            onclick={() => add(gid)}
          >
            {gid}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
  <div class="row-actions">
    <button
      class="btn btn-primary"
      type="button"
      data-testid={`groups-save-${uid}`}
      onclick={() => void save()}
    >
      Valider
    </button>
    <button
      class="btn"
      type="button"
      data-testid={`groups-cancel-${uid}`}
      onclick={onCancel}
    >
      Annuler
    </button>
  </div>
</div>
