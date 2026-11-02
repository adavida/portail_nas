<script lang="ts" module>
  export type CellObj = { text: string; cls?: string; href?: string; testid?: string };
  export type Cell = string | CellObj | Cell[] | null;
  export type Row = { testid?: string; cls?: string; cells: Cell[]; onclick?: () => void };
</script>

<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    testid?: string;
    head: string[];
    empty: string;
    emptyTestid: string;
    isEmpty: boolean;
    cls?: string;
    rows?: Snippet;
    data?: Row[];
  };

  let { testid, head, empty, emptyTestid, isEmpty, cls = "", rows, data }: Props = $props();

  function td_testid(cell: Cell): string | undefined {
    return typeof cell === "object" && cell !== null && !Array.isArray(cell)
      ? cell.testid
      : undefined;
  }
</script>

{#snippet cell_view(cell: Cell)}
  {#if Array.isArray(cell)}
    {#each cell as sub, k (k)}
      {@render cell_view(sub)}
    {/each}
  {:else if cell === null}
    <span class="muted">—</span>
  {:else if typeof cell === "string"}
    {cell}
  {:else if cell.href}
    <a href={cell.href} class={cell.cls}>{cell.text}</a>
  {:else}
    <span class={cell.cls}>{cell.text}</span>
  {/if}
{/snippet}

<div class="table-scroll">
  <table data-testid={testid} class="table {cls}">
    <thead>
      <tr>
        {#each head as label (label)}
          <th>{label}</th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#if isEmpty}
        <tr>
          <td colSpan={head.length} class="center" data-testid={emptyTestid}>{empty}</td>
        </tr>
      {:else if data}
        {#each data as row, i (row.testid ?? i)}
          <tr
            data-testid={row.testid}
            class={row.cls ?? ""}
            tabindex={row.onclick ? 0 : undefined}
            onclick={row.onclick}
            onkeydown={(e) => e.key === "Enter" && row.onclick?.()}
          >
            {#each row.cells as cell, j (j)}
              <td data-testid={td_testid(cell)}>
                {@render cell_view(cell)}
              </td>
            {/each}
          </tr>
        {/each}
      {:else if rows}
        {@render rows()}
      {/if}
    </tbody>
  </table>
</div>
