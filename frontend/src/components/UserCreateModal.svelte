<script lang="ts">
  import { createUser } from "../api/users";
  import { t, translate } from "../i18n";
  import { toast } from "./toast";

  type Props = {
    onClose: () => void;
    onCreated?: () => void;
  };

  let { onClose, onCreated }: Props = $props();

  let uid = $state("");
  let name = $state("");
  let email = $state("");
  let password = $state("");

  function onWindowKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }

  async function submit() {
    if (!uid.trim() || !name.trim() || !password) {
      toast(translate("common.fields_required"), true);
      return;
    }
    const created = uid;
    try {
      await createUser({ uid, name, email, password });
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    onCreated?.();
    onClose();
    toast(translate("users.created", { uid: created }));
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

<div
  class="modal-backdrop"
  data-testid="user-create-modal"
  role="dialog"
  aria-modal="true"
  aria-label={$t("modal.user_create")}
>
  <div class="modal card">
    <div class="modal-header">
      <h3>{$t("modal.user_new")}</h3>
      <button
        class="btn"
        type="button"
        aria-label={$t("nav.close")}
        data-testid="create-close"
        onclick={onClose}
      >
        ×
      </button>
    </div>

    <label class="label">
      UID
      <input class="input" data-testid="input-uid" bind:value={uid} />
    </label>
    <label class="label">
      {$t("field.name")}
      <input class="input" data-testid="input-name" bind:value={name} />
    </label>
    <label class="label">
      {$t("field.email")}
      <input class="input" data-testid="input-email" bind:value={email} />
    </label>
    <label class="label">
      {$t("field.password")}
      <input
        class="input"
        type="password"
        data-testid="input-password"
        bind:value={password}
      />
    </label>

    <div class="row-actions">
      <button
        class="btn btn-primary"
        type="button"
        data-testid="create-button"
        onclick={() => void submit()}
      >
        {$t("common.validate")}
      </button>
      <button
        class="btn"
        type="button"
        data-testid="create-cancel"
        onclick={onClose}
      >
        {$t("common.cancel")}
      </button>
    </div>
  </div>
</div>
