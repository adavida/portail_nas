<script lang="ts">
  import { t } from "../i18n";

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
    {$t("common.validate")}
  </button>
  <button
    class="btn"
    type="button"
    data-testid={`${id}-cancel`}
    onclick={onCancel}
  >
    {$t("common.cancel")}
  </button>
</div>
