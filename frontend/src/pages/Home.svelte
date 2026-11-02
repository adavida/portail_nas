<script lang="ts">
  import { onMount } from "svelte";
  import { listApps, type App } from "../api/apps";
  import { t, translate } from "../i18n";
  import { toast } from "../components/toast";
  import "./Home.scss";

  let { groups = [] } = $props();
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

  let visible = $derived(
    apps?.filter(
      (app) =>
        (!app.required_group || groups.includes(app.required_group)) &&
        (!app.forbidden_group || !groups.includes(app.forbidden_group)),
    ),
  );
</script>

<section>
  <h2>{$t("apps.title")}</h2>
  {#if error}
    <p data-testid="apps-error">{$t("common.error", { error: error ?? "" })}</p>
  {:else if visible === undefined}
    <p data-testid="apps-loading">{$t("common.loading")}</p>
  {:else if visible.length === 0}
    <p data-testid="apps-empty">{$t("apps.empty")}</p>
  {:else}
    <ul class="apps">
      {#each visible as app, i (`${app.name}-${app.url}`)}
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
