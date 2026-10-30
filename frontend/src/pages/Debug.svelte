<script lang="ts">
  import { onMount } from "svelte";
  import { getClaims, getDebug, type DebugInfo } from "../api/debug";
  import { listApps, type App } from "../api/apps";
  import { listUsers, type User } from "../api/users";
  import { t } from "../i18n";

  let info = $state<DebugInfo | null>(null);
  let apps = $state<App[] | null>(null);
  let users = $state<User[]>([]);
  let error = $state<string | null>(null);

  let selectedUid = $state("");
  let selectedClient = $state("");
  let claims = $state<string | null>(null);
  let claimsError = $state<string | null>(null);
  let claimsLoading = $state(false);

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

  function showClaims() {
    if (!selectedUid || !selectedClient) return;
    claimsLoading = true;
    claimsError = null;
    claims = null;
    getClaims(selectedUid, selectedClient)
      .then((c) => (claims = JSON.stringify(c, null, 2)))
      .catch((e: unknown) => {
        claimsError = e instanceof Error ? e.message : String(e);
      })
      .finally(() => (claimsLoading = false));
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
      <h3>{$t("debug.apps")}</h3>
      {#if apps.length === 0}
        <p data-testid="debug-apps-empty">{$t("apps.empty")}</p>
      {:else}
        <ul>
          {#each apps as app, i (app.name)}
            <li data-testid="debug-app-{i}">{app.name} — {app.url}</li>
          {/each}
        </ul>
      {/if}
    </div>
    <div class="card">
      <h3>{$t("debug.oidc")}</h3>
      <dl data-testid="debug-oidc">
        <dt>issuer</dt>
        <dd>{info.issuer}</dd>
        <dt>key_bits</dt>
        <dd>{info.key_bits}</dd>
        <dt>pending_ttl_secs</dt>
        <dd>{info.pending_ttl_secs}</dd>
        <dt>code_ttl_secs</dt>
        <dd>{info.code_ttl_secs}</dd>
        <dt>session_ttl_secs</dt>
        <dd>{info.session_ttl_secs}</dd>
        <dt>access_ttl_secs</dt>
        <dd>{info.access_ttl_secs}</dd>
      </dl>
      <table>
        <thead>
          <tr>
            <th>id</th>
            <th>enabled</th>
            <th>has_secret</th>
            <th>redirect_uris</th>
            <th>scopes</th>
            <th>required_group</th>
            <th>forbidden_group</th>
          </tr>
        </thead>
        <tbody>
          {#each info.clients as client, i (client.id)}
            <tr data-testid="debug-client-{i}">
              <td>{client.id}</td>
              <td>{client.enabled}</td>
              <td>{client.has_secret}</td>
              <td>{client.redirect_uris.join(", ")}</td>
              <td>{client.scopes.join(", ")}</td>
              <td>{client.required_group || "—"}</td>
              <td>{client.forbidden_group || "—"}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      <h4>{$t("debug.claims")}</h4>
      <div class="claims-picker">
        <label>
          {$t("debug.user")}
          <select data-testid="claims-user" bind:value={selectedUid} onchange={showClaims}>
            <option value="" disabled>—</option>
            {#each users as user (user.uid)}
              <option value={user.uid}>{user.uid} ({user.name})</option>
            {/each}
          </select>
        </label>
        <label>
          {$t("debug.client")}
          <select
            data-testid="claims-client"
            bind:value={selectedClient}
            onchange={showClaims}
          >
            <option value="" disabled>—</option>
            {#each info.clients as client (client.id)}
              <option value={client.id}>{client.id}</option>
            {/each}
          </select>
        </label>
      </div>
      {#if claimsLoading}
        <p data-testid="claims-loading">{$t("common.loading")}</p>
      {:else if claimsError}
        <p data-testid="claims-error">
          {$t("common.error", { error: claimsError })}
        </p>
      {:else if claims !== null}
        <pre data-testid="debug-claims">{claims}</pre>
      {/if}
    </div>
  {/if}
</section>
