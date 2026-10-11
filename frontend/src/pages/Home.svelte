<script lang="ts">
  import { onMount } from "svelte";
  import { listApps, type App } from "../api/apps";
  import { toast } from "../components/toast";
  import "./Home.scss";

  let apps = $state<App[] | null>(null);
  let error = $state<string | null>(null);

  function fetchApps() {
    listApps()
      .then((a) => (apps = a))
      .catch(() => {
        error = "error";
        toast("Erreur: chargement des applications impossible", true);
      });
  }

  onMount(fetchApps);
</script>

<section>
  <h2>Applications</h2>
  {#if error}
    <p data-testid="apps-error">Erreur: {error}</p>
  {:else if apps === null}
    <p data-testid="apps-loading">Chargement...</p>
  {:else if apps.length === 0}
    <p data-testid="apps-empty">Aucune application</p>
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
