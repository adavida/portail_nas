<script lang="ts">
  import "./LabeledInput.scss";

  type Props = {
    label: string;
    value: string;
    testId: string;
    type?: "text" | "email" | "password" | "number" | "tel";
    placeholder?: string;
    maxLength?: number | null;
    min?: number | null;
    max?: number | null;
  };

  let {
    label,
    value = $bindable(),
    testId,
    type = "text",
    placeholder = "",
    maxLength = null,
    min = null,
    max = null,
  }: Props = $props();
</script>

<label class="labeled-input">
  <span>{label}</span>
  <input
    class="input"
    {type}
    {placeholder}
    maxlength={maxLength ?? undefined}
    min={min ?? undefined}
    max={max ?? undefined}
    data-testid={testId}
    bind:value
  />
</label>
