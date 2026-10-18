<script lang="ts">
  import { loginWithPassword } from "../api/oidc";
  import ThemeLangControls from "../components/ThemeLangControls.svelte";
  import { t, translate } from "../i18n";
  import "./Login.scss";

  // The provider 302s here with ?req=<flow id>; without it there is nothing to post.
  const req = new URLSearchParams(window.location.search).get("req") ?? "";

  let username = $state("");
  let password = $state("");
  let pending = $state(false);
  let error = $state<string | null>(req ? null : translate("login.missing_request"));

  async function submit() {
    if (!req || pending) return;
    pending = true;
    error = null;
    try {
      const r = await loginWithPassword(req, username, password);
      if ("redirect" in r) {
        // Keep `pending` true: the button stays disabled while navigating away.
        window.location.href = r.redirect;
        return;
      }
      // 401 = wrong credentials (translated); anything else is a generic error.
      error = r.status === 401 ? translate("login.invalid") : r.error;
    } catch (e) {
      error = (e as Error).message;
    }
    pending = false;
  }
</script>

<div class="login">
  <form
    class="card login-card"
    data-testid="login-form"
    onsubmit={(e) => {
      e.preventDefault();
      void submit();
    }}
  >
    <h3>{$t("login.title")}</h3>

    <label class="label">
      {$t("login.username")}
      <input
        class="input"
        autocomplete="username"
        aria-invalid={error !== null}
        data-testid="input-username"
        bind:value={username}
      />
    </label>
    <label class="label">
      {$t("login.password")}
      <input
        class="input"
        type="password"
        autocomplete="current-password"
        aria-invalid={error !== null}
        data-testid="input-password"
        bind:value={password}
      />
    </label>

    {#if error}
      <p class="login-error" role="alert" data-testid="login-error">{error}</p>
    {/if}

    <button
      class="btn btn-primary"
      type="submit"
      data-testid="login-button"
      disabled={pending || !req}
    >
      {$t("login.submit")}
    </button>
  </form>

  <div class="login-settings">
    <ThemeLangControls />
  </div>
</div>
