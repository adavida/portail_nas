<script lang="ts">
  import { onMount } from "svelte";
  import { getDebug, type DebugInfo } from "../api/debug";
  import { listApps, type App } from "../api/apps";
  import { listUsers, type User } from "../api/users";
  import { t } from "../i18n";
  import DebugApps from "../components/DebugApps.svelte";
  import DebugClaims from "../components/DebugClaims.svelte";
  import DebugOidc from "../components/DebugOidc.svelte";

  let info = $state<DebugInfo | null>(null);
  let apps = $state<App[] | null>(null);
  let users = $state<User[]>([]);
  let error = $state<string | null>(null);

  function fetchDebug() {
    Promise.all([getDebug(), listApps(), listUsers()])
      .then(([i, a, u]) => {
        info = i;
        apps = a;
        users = u;
      })
      .catch((e: unknown) => {
        error = e instanceof Error ? e.message : String(e);
      });
  }

  onMount(fetchDebug);
</script>

<section>
  <h2>{$t("nav.debug")}</h2>
  {#if error}
    <p data-testid="debug-error">{$t("common.error", { error: error ?? "" })}</p>
  {:else if info === null || apps === null}
    <p data-testid="debug-loading">{$t("common.loading")}</p>
  {:else}
    <div class="card">
      <h3>{$t("debug.oidc")}</h3>
      <DebugApps {apps} clients={info.clients} />
      <DebugOidc {info} />
      <DebugClaims {users} clients={info.clients} />
    </div>
  {/if}
</section>
