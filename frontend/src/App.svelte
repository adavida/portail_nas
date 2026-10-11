<script lang="ts">
  import Callback from "./pages/Callback.svelte";
  import Groups from "./pages/Groups.svelte";
  import Home from "./pages/Home.svelte";
  import Users from "./pages/Users.svelte";
  import Toaster from "./components/Toaster.svelte";
  import { whoami } from "./api/auth";
  import { clearTokens, getToken, login, logout } from "./auth/oidc";
  import "./App.scss";

  type Theme = "light" | "dark";

  const isCallback = window.location.pathname === "/callback";

  function initialTheme(): Theme {
    const saved = localStorage.getItem("theme");
    if (saved === "light" || saved === "dark") return saved;
    return window.matchMedia?.("(prefers-color-scheme: dark)")?.matches
      ? "dark"
      : "light";
  }

  let tab = $state<"home" | "users" | "groups">("home");
  let checked = $state(false);
  let admin = $state(false);
  let theme = $state<Theme>(initialTheme());
  const token = getToken();

  $effect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("theme", theme);
  });

  $effect(() => {
    if (isCallback) return;
    if (sessionStorage.getItem("login_in_progress") === "1") return;
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
{:else if !token}
  {#if sessionStorage.getItem("logged_out") === "1"}
    <div class="logged-out">
      <p data-testid="logged-out">Déconnecté</p>
      <button
        class="btn btn-primary"
        data-testid="login-button"
        onclick={() => {
          sessionStorage.removeItem("logged_out");
          sessionStorage.setItem("login_in_progress", "1");
          void login();
        }}
      >
        Se connecter
      </button>
    </div>
  {:else}
    <p class="muted" data-testid="redirecting">Redirection vers Authelia...</p>
  {/if}
{:else if !checked}
  <p class="muted" data-testid="checking">Vérification...</p>
{:else}
  <div class="layout">
    <aside class="sidebar">
      <div class="brand">
        <span class="brand-badge">P</span>
        <span>Portail</span>
      </div>
      <nav class="nav">
        <button
          class={tab === "home" ? "nav-item active" : "nav-item"}
          data-testid="tab-home"
          onclick={() => (tab = "home")}
        >
          Accueil
        </button>
        {#if admin}
          <button
            class={tab === "users" ? "nav-item active" : "nav-item"}
            data-testid="tab-users"
            onclick={() => (tab = "users")}
          >
            Utilisateurs
          </button>
          <button
            class={tab === "groups" ? "nav-item active" : "nav-item"}
            data-testid="tab-groups"
            onclick={() => (tab = "groups")}
          >
            Groupes
          </button>
        {/if}
      </nav>
      <div class="sidebar-spacer"></div>
      <button
        class="nav-item theme-toggle"
        role="switch"
        aria-checked={theme === "dark"}
        data-testid="theme-toggle"
        onclick={() => (theme = theme === "dark" ? "light" : "dark")}
      >
        <span>{theme === "dark" ? "Thème sombre" : "Thème clair"}</span>
        <span class="theme-toggle-track" aria-hidden="true">
          <span class="theme-toggle-thumb"></span>
        </span>
      </button>
      <button class="nav-item logout" data-testid="logout" onclick={logout}>
        Déconnexion
      </button>
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
