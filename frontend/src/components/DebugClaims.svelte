<script lang="ts">
  import { getClaims, type DebugClient } from "../api/debug";
  import type { User } from "../api/users";
  import { t } from "../i18n";

  type Props = {
    users: User[];
    clients: DebugClient[];
  };

  let { users, clients }: Props = $props();

  let selectedUid = $state("");
  let selectedClient = $state("");
  let claims = $state<string | null>(null);
  let claimsError = $state<string | null>(null);
  let claimsLoading = $state(false);

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
</script>

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
    <select data-testid="claims-client" bind:value={selectedClient} onchange={showClaims}>
      <option value="" disabled>—</option>
      {#each clients as client (client.id)}
        <option value={client.id}>{client.id}</option>
      {/each}
    </select>
  </label>
</div>
{#if claimsLoading}
  <p data-testid="claims-loading">{$t("common.loading")}</p>
{:else if claimsError}
  <p data-testid="claims-error">{$t("common.error", { error: claimsError })}</p>
{:else if claims !== null}
  <pre data-testid="debug-claims">{claims}</pre>
{/if}
