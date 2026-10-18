<script lang="ts">
  import { t } from "../i18n";
  import { theme } from "../theme";

  function toggleTheme() {
    theme.update((current) => (current === "dark" ? "light" : "dark"));
  }
</script>

<button
  class="nav-item theme-toggle"
  role="switch"
  aria-checked={$theme === "dark"}
  data-testid="theme-toggle"
  onclick={toggleTheme}
>
  <span>{$theme === "dark" ? $t("nav.theme_dark") : $t("nav.theme_light")}</span>
  <span class="theme-toggle-track" aria-hidden="true">
    <span class="theme-toggle-thumb"></span>
  </span>
</button>
