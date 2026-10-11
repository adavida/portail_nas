<script lang="ts">
  import { createUser } from "../api/users";
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
      toast("uid, nom et mot de passe requis", true);
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
    toast(`Utilisateur ${created} créé`);
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

<div
  class="modal-backdrop"
  data-testid="user-create-modal"
  role="dialog"
  aria-modal="true"
  aria-label="Création utilisateur"
>
  <div class="modal card">
    <div class="modal-header">
      <h3>Nouvel utilisateur</h3>
      <button
        class="btn"
        type="button"
        aria-label="Fermer"
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
      Nom
      <input class="input" data-testid="input-name" bind:value={name} />
    </label>
    <label class="label">
      Email
      <input class="input" data-testid="input-email" bind:value={email} />
    </label>
    <label class="label">
      Mot de passe
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
        Valider
      </button>
      <button
        class="btn"
        type="button"
        data-testid="create-cancel"
        onclick={onClose}
      >
        Annuler
      </button>
    </div>
  </div>
</div>
