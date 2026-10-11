<script lang="ts">
  import { onMount } from "svelte";
  import { handleCallback } from "../auth/oidc";
  import { t } from "../i18n";

  let error = $state<string | null>(null);

  function done() {
    window.location.replace("/");
  }

  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    const code = params.get("code");
    const state = params.get("state");
    const storedState = sessionStorage.getItem("oidc_state");
    if (state && storedState && state !== storedState) {
      error = `state mismatch: ${state} vs ${storedState}`;
      return;
    }
    if (!code) {
      const err = params.get("error");
      const desc = params.get("error_description");
      error = err ? `${err}: ${desc ?? ""}` : "code missing";
      return;
    }
    const key = `handled_${code}`;
    if (sessionStorage.getItem(key)) return;
    sessionStorage.setItem(key, "1");
    if (localStorage.getItem("access_token")) {
      sessionStorage.removeItem("login_in_progress");
      done();
      return;
    }
    handleCallback(code)
      .then(() => {
        sessionStorage.removeItem("login_in_progress");
        sessionStorage.removeItem("logged_out");
        sessionStorage.removeItem("oidc_state");
        sessionStorage.removeItem("oidc_verifier");
        done();
      })
      .catch((e) => {
        sessionStorage.removeItem("login_in_progress");
        error = String(e);
      });
  });
</script>

{#if error}
  <p data-testid="callback-error">{$t("common.error", { error: error ?? "" })}</p>
{:else}
  <p data-testid="callback-loading">{$t("auth.connecting")}</p>
{/if}
