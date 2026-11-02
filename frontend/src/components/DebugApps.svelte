<script lang="ts">
  import type { App } from "../api/apps";
  import type { DebugClient } from "../api/debug";
  import { t } from "../i18n";
  import Table, { type Cell } from "./Table.svelte";

  type Props = {
    apps: App[];
    clients: DebugClient[];
  };

  let { apps, clients }: Props = $props();

  function client_cells(client: DebugClient | null): Cell[] {
    if (!client) {
      return [null, { text: "no client", cls: "state" }, null, null, null, null, null];
    }
    return [
      [{ text: client.id, cls: "tag" }],
      {
        text: client.enabled ? "enabled" : "disabled",
        cls: `state ${client.enabled ? "ok" : "off"}`,
      },
      {
        text: client.has_secret ? "yes" : "no",
        cls: `state ${client.has_secret ? "ok" : ""}`,
      },
      client.redirect_uris.map((uri) => ({ text: uri, cls: "tag" })),
      client.scopes.map((scope) => ({ text: scope, cls: "tag" })),
      client.required_group ? { text: client.required_group, cls: "state ok" } : null,
      client.forbidden_group ? { text: client.forbidden_group, cls: "state off" } : null,
    ];
  }

  const debug_rows = $derived([
    ...apps.map((app, i) => ({
      testid: `debug-app-${i}`,
      cells: [
        { text: app.name, cls: "name" },
        { text: app.url, cls: "url", href: app.url },
        ...client_cells(clients.find((c) => c.id === app.name.toLowerCase()) ?? null),
      ],
    })),
    ...clients
      .filter((c) => !apps.some((a) => a.name.toLowerCase() === c.id))
      .map((c, i) => ({
        testid: `debug-client-${i}`,
        cells: [{ text: c.id, cls: "name" }, null, ...client_cells(c)],
      })),
  ]);
</script>

<h4>{$t("debug.apps")}</h4>
<Table
  cls="debug-table"
  head={[
    $t("debug.apps"),
    "URL",
    "client",
    "status",
    "secret",
    "redirect_uris",
    "scopes",
    "required_group",
    "forbidden_group",
  ]}
  empty={$t("apps.empty")}
  emptyTestid="debug-apps-empty"
  isEmpty={debug_rows.length === 0}
  data={debug_rows}
/>
