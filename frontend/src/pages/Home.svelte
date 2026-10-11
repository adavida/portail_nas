<script lang="ts">
  import { onMount } from "svelte";
  import { listApps, type App } from "../api/apps";
  import { t, translate } from "../i18n";
  import { toast } from "../components/toast";
  import "./Home.scss";

  let apps = $state<App[] | null>(null);
  let error = $state<string | null>(null);

  function fetchApps() {
    listApps()
      .then((a) => (apps = a))
      .catch(() => {
        error = "error";
        toast(translate("apps.error_loading"), true);
      });
  }

  onMount(fetchApps);
</script>

<section>
  <h2>{$t("apps.title")}</h2>
  {#if error}
    <p data-testid="apps-error">{$t("common.error", { error: error ?? "" })}</p>
  {:else if apps === null}
    <p data-testid="apps-loading">{$t("common.loading")}</p>
  {:else if apps.length === 0}
    <p data-testid="apps-empty">{$t("apps.empty")}</p>
  {:else}
    <ul class="apps">
      {#each apps as app, i (`${app.name}-${app.url}`)}
        <li data-testid={`app-${i}`}>
          <a
            class="app-link"
            data-testid={`app-link-${i}`}
            href={app.url}
            target="_blank"
            rel="noopener"
          >
            {#if app.icon}
              <img src={app.icon} width="20" height="20" alt="" />
            {/if}
            {app.name}
          </a>
          {#if app.description}
            <p class="app-desc">{app.description}</p>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>
