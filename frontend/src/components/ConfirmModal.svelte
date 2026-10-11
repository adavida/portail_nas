<script lang="ts">
  import { t } from "../i18n";

  type Props = {
    message: string;
    onConfirm: () => void;
    onCancel: () => void;
  };

  let { message, onConfirm, onCancel }: Props = $props();

  function onWindowKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onCancel();
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

<div
  class="modal-backdrop"
  role="alertdialog"
  aria-modal="true"
  aria-label={$t("modal.confirm_delete")}
  data-testid="confirm-modal"
>
  <div class="modal card">
    <div class="modal-header">
      <h3>{message}</h3>
    </div>
    <div class="row-actions">
      <button
        class="btn btn-danger"
        type="button"
        data-testid="confirm-yes"
        onclick={onConfirm}
      >
        {$t("common.delete")}
      </button>
      <button
        class="btn"
        type="button"
        data-testid="confirm-no"
        onclick={onCancel}
      >
        {$t("common.cancel")}
      </button>
    </div>
  </div>
</div>
