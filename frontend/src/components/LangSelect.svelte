<script lang="ts">
  import { lang, t } from "../i18n";
</script>

<select
  class="lang-select"
  data-testid="lang-select"
  aria-label={$t("nav.lang")}
  bind:value={$lang}
>
  <option value="fr">🇫🇷 FR</option>
  <option value="en">🇺🇸 EN</option>
</select>
