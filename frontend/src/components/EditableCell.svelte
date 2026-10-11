<script lang="ts">
  type Props = {
    value: string;
    field: "name" | "email" | "description";
    rowId: string;
    onSave: (field: string, value: string) => Promise<boolean>;
  };

  let { value, field, rowId, onSave }: Props = $props();

  let editing = $state(false);
  let draft = $state("");

  async function save() {
    if (await onSave(field, draft)) editing = false;
  }

  function startEditing() {
    draft = value;
    editing = true;
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter") void save();
    if (e.key === "Escape") editing = false;
  }
</script>

{#if !editing}
  <td
    data-testid={`cell-${field}-${rowId}`}
    class="editable"
    ondblclick={startEditing}
  >
    {value}
  </td>
{:else}
  <td>
    <!-- svelte-ignore a11y_autofocus -->
    <input
      class="input"
      data-testid={`edit-input-${field}-${rowId}`}
      autofocus
      bind:value={draft}
      onblur={() => void save()}
      onkeydown={onKeyDown}
    />
  </td>
{/if}
