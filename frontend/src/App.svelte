<script lang="ts">
  import Callback from "./pages/Callback.svelte";
  import Directory from "./pages/Directory.svelte";
  import Home from "./pages/Home.svelte";
  import Login from "./pages/Login.svelte";
  import Monitoring from "./pages/Monitoring.svelte";
  import Profile from "./pages/Profile.svelte";
  import Toaster from "./components/Toaster.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import { whoami } from "./api/auth";
  import { clearTokens, getToken, login } from "./auth/oidc";
  import { t } from "./i18n";
  import "./App.scss";

  const isCallback = window.location.pathname === "/callback";
  const isLogin = window.location.pathname === "/login";

  let tab = $state<"home" | "profile" | "directory" | "monitoring">("home");
  let checked = $state(false);
  let admin = $state(false);
  let groups = $state<string[]>([]);
  const token = getToken();

  $effect(() => {
    if (isCallback || isLogin) return;
    if (!token) {
      if (sessionStorage.getItem("logged_out") === "1") return;
      sessionStorage.setItem("login_in_progress", "1");
      void login();
      return;
    }
    void whoami().then((s) => {
      if (s === "relogin") {
        clearTokens();
        sessionStorage.setItem("login_in_progress", "1");
        void login();
        return;
      }
      admin = s.is_admin;
      groups = s.groups;
      checked = true;
      sessionStorage.removeItem("login_in_progress");
    });
  });
</script>

{#if isCallback}
  <Callback />
{:else if isLogin}
  <Login />
{:else if !token}
  {#if sessionStorage.getItem("logged_out") === "1"}
    <div class="logged-out">
      <p data-testid="logged-out">{$t("auth.logged_out")}</p>
      <button
        class="btn btn-primary"
        data-testid="login-button"
        onclick={() => {
          sessionStorage.removeItem("logged_out");
          sessionStorage.setItem("login_in_progress", "1");
          void login();
        }}
      >
        {$t("auth.login")}
      </button>
    </div>
  {:else}
    <p class="muted" data-testid="redirecting">{$t("auth.redirecting")}</p>
  {/if}
{:else if !checked}
  <p class="muted" data-testid="checking">{$t("auth.checking")}</p>
{:else}
  <div class="layout">
    <Sidebar {admin} bind:tab />
    <main class="content">
      {#if tab === "home"}
        <Home {groups} />
      {:else if tab === "profile"}
        <Profile />
      {:else if tab === "directory"}
        <Directory />
      {:else}
        <Monitoring />
      {/if}
    </main>
    <Toaster />
  </div>
{/if}
