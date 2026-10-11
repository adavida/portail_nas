<script lang="ts">
  type Props = {
    label: string;
    id: string;
    value: string;
    onValidate: () => void;
    onCancel: () => void;
  };

  let {
    label,
    id,
    value = $bindable(),
    onValidate,
    onCancel,
  }: Props = $props();
</script>

<label class="label">
  {label}
  <input class="input" data-testid={id} bind:value={value} />
</label>
<div class="row-actions">
  <button
    class="btn btn-primary"
    type="button"
    data-testid={`${id}-save`}
    onclick={onValidate}
  >
    Valider
  </button>
  <button
    class="btn"
    type="button"
    data-testid={`${id}-cancel`}
    onclick={onCancel}
  >
    Annuler
  </button>
</div>
