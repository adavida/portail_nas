<script lang="ts">
  import Callback from "./pages/Callback.svelte";
  import Debug from "./pages/Debug.svelte";
  import Groups from "./pages/Groups.svelte";
  import Home from "./pages/Home.svelte";
  import Login from "./pages/Login.svelte";
  import Users from "./pages/Users.svelte";
  import Toaster from "./components/Toaster.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import { whoami } from "./api/auth";
  import { clearTokens, getToken, login } from "./auth/oidc";
  import { t } from "./i18n";
  import "./App.scss";

  const isCallback = window.location.pathname === "/callback";
  const isLogin = window.location.pathname === "/login";

  let tab = $state<"home" | "users" | "groups" | "debug">("home");
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
      {:else if tab === "users"}
        <Users />
      {:else if tab === "groups"}
        <Groups />
      {:else}
        <Debug />
      {/if}
    </main>
    <Toaster />
  </div>
{/if}
