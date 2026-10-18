<script lang="ts">
  import Callback from "./pages/Callback.svelte";
  import Groups from "./pages/Groups.svelte";
  import Home from "./pages/Home.svelte";
  import Login from "./pages/Login.svelte";
  import Users from "./pages/Users.svelte";
  import Toaster from "./components/Toaster.svelte";
  import ThemeLangControls from "./components/ThemeLangControls.svelte";
  import { whoami } from "./api/auth";
  import { clearTokens, getToken, login, logout } from "./auth/oidc";
  import { t } from "./i18n";
  import "./App.scss";

  const isCallback = window.location.pathname === "/callback";
  const isLogin = window.location.pathname === "/login";

  let tab = $state<"home" | "users" | "groups">("home");
  let menuOpen = $state(false);
  let checked = $state(false);
  let admin = $state(false);
  const token = getToken();

  $effect(() => {
    // isLogin: never auto-login here, otherwise authorize → /login → effect →
    // login() loops forever.
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
      admin = s === "admin";
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
    <aside class="sidebar" class:open={menuOpen}>
      <div class="brand">
        <span class="brand-badge">P</span>
        <span>Portail</span>
      </div>
      <button
        class="nav-item nav-burger"
        type="button"
        aria-label={$t("nav.menu")}
        aria-expanded={menuOpen}
        data-testid="nav-burger"
        onclick={() => (menuOpen = !menuOpen)}
      >
        {menuOpen ? "✕" : "☰"}
      </button>
      <div class="sidebar-menu">
        <nav class="nav">
          <button
            class={tab === "home" ? "nav-item active" : "nav-item"}
            data-testid="tab-home"
            onclick={() => {
              tab = "home";
              menuOpen = false;
            }}
          >
            {$t("nav.home")}
          </button>
          {#if admin}
            <button
              class={tab === "users" ? "nav-item active" : "nav-item"}
              data-testid="tab-users"
              onclick={() => {
                tab = "users";
                menuOpen = false;
              }}
            >
              {$t("nav.users")}
            </button>
            <button
              class={tab === "groups" ? "nav-item active" : "nav-item"}
              data-testid="tab-groups"
              onclick={() => {
                tab = "groups";
                menuOpen = false;
              }}
            >
              {$t("nav.groups")}
            </button>
          {/if}
        </nav>
        <div class="sidebar-spacer"></div>
        <ThemeLangControls />
        <button class="nav-item logout" data-testid="logout" onclick={logout}>
          {$t("nav.logout")}
        </button>
      </div>
    </aside>
    <main class="content">
      {#if tab === "home"}
        <Home />
      {:else if tab === "users"}
        <Users />
      {:else}
        <Groups />
      {/if}
    </main>
    <Toaster />
  </div>
{/if}
