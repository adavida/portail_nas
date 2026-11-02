<script lang="ts">
  import ThemeLangControls from "./ThemeLangControls.svelte";
  import { logout } from "../auth/oidc";
  import { t } from "../i18n";

  type Tab = "home" | "users" | "groups" | "debug";

  let { tab = $bindable<Tab>("home"), admin = false } = $props();
  let menuOpen = $state(false);

  const items: { id: Tab; labelKey: string; testId: string; adminOnly: boolean }[] =
    [
      { id: "home", labelKey: "nav.home", testId: "tab-home", adminOnly: false },
      { id: "users", labelKey: "nav.users", testId: "tab-users", adminOnly: true },
      { id: "groups", labelKey: "nav.groups", testId: "tab-groups", adminOnly: true },
      { id: "debug", labelKey: "nav.debug", testId: "tab-debug", adminOnly: true },
    ];

  function select(next: Tab) {
    tab = next;
    menuOpen = false;
  }
</script>

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
      {#each items as item (item.id)}
        {#if !item.adminOnly || admin}
          <button
            class={tab === item.id ? "nav-item active" : "nav-item"}
            data-testid={item.testId}
            onclick={() => select(item.id)}
          >
            {$t(item.labelKey)}
          </button>
        {/if}
      {/each}
    </nav>
    <div class="sidebar-spacer"></div>
    <ThemeLangControls />
    <button class="nav-item logout" data-testid="logout" onclick={logout}>
      {$t("nav.logout")}
    </button>
  </div>
</aside>
