<script lang="ts">
  import type { DebugInfo } from "../api/debug";
  import Table, { type Cell } from "./Table.svelte";

  type Props = {
    info: DebugInfo;
  };

  let { info }: Props = $props();

  const oidc_rows = $derived(
    (
      [
        ["issuer", info.issuer],
        ["key_bits", String(info.key_bits)],
        ["pending_ttl_secs", String(info.pending_ttl_secs)],
        ["code_ttl_secs", String(info.code_ttl_secs)],
        ["session_ttl_secs", String(info.session_ttl_secs)],
        ["access_ttl_secs", String(info.access_ttl_secs)],
      ] as [string, string][]
    ).map(([key, value]) => ({
      testid: `oidc-${key}`,
      cells: [{ text: key, cls: "name" }, value] as Cell[],
    })),
  );
</script>

<Table
  testid="debug-oidc"
  cls="param-table"
  head={["param", "value"]}
  empty=""
  emptyTestid="oidc-empty"
  isEmpty={false}
  data={oidc_rows}
/>
