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
  const matches = $derived(
    q
      ? allGroups
          .filter(
            (gid) => !draft.includes(gid) && gid.toLowerCase().includes(q),
          )
          .sort(
            (a, b) =>
              Number(b.toLowerCase().startsWith(q)) -
              Number(a.toLowerCase().startsWith(q)),
          )
      : [],
  );
  const exact = $derived(
    q !== "" && allGroups.some((g) => g.toLowerCase() === q),
  );
  type Option = { kind: "create" } | { kind: "add"; gid: string };
  const options = $derived<Option[]>([
    ...(q && !exact ? [{ kind: "create" as const }] : []),
    ...matches.map((gid) => ({ kind: "add" as const, gid })),
  ]);
  let active = $state(0);
  const activeIdx = $derived(Math.min(active, options.length - 1));
  let listEl = $state<HTMLUListElement | null>(null);

  $effect(() => {
    if (activeIdx >= 0) {
      listEl?.querySelector(".active")?.scrollIntoView?.({ block: "nearest" });
    }
  });
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
    if (existing) {
      add(existing);
      return;
    }
    if (!onCreateGroup) return;
    const res = await onCreateGroup(gid);
    if (res.ok) add(gid);
  }

  async function handleEnter() {
    if (!newGid.trim()) {
      await save();
      return;
    }
    const opt = options[activeIdx];
    if (!opt) {
      newGid = "";
      return;
    }
    if (opt.kind === "create") await resolveAndAdd(newGid);
    else add(opt.gid);
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      void handleEnter();
    } else if (e.key === "ArrowDown" && options.length) {
      e.preventDefault();
      active = (active + 1) % options.length;
    } else if (e.key === "ArrowUp" && options.length) {
      e.preventDefault();
      active = (active - 1 + options.length) % options.length;
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
      oninput={() => (active = 0)}
      onkeydown={onKeyDown}
    />
    {#if options.length > 0}
      <ul
        bind:this={listEl}
        data-testid={`groups-suggest-list-${uid}`}
        class="suggest"
      >
        {#each options as opt, i (opt.kind === "create" ? "create" : opt.gid)}
          <li>
            <button
              type="button"
              class="opt"
              class:active={i === activeIdx}
              class:muted={opt.kind === "create"}
              data-testid={opt.kind === "create"
                ? `groups-create-${uid}`
                : `groups-suggest-${uid}-${opt.gid}`}
              onclick={() =>
                opt.kind === "create"
                  ? void resolveAndAdd(newGid)
                  : add(opt.gid)}
            >
              {opt.kind === "create" ? `créer « ${newGid.trim()} »` : opt.gid}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
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
