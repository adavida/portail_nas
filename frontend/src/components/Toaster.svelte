<script lang="ts">
  import { toasts } from "./toast";
  import "./Toaster.scss";
</script>

{#if $toasts.length > 0}
  <div data-testid="toaster" class="toaster">
    {#each $toasts as t (t.id)}
      <div
        class={t.isError ? "toast toast--error" : "toast"}
        data-testid={t.isError ? "toast-error" : "toast"}
      >
        {t.message}
      </div>
    {/each}
  </div>
{/if}
