<script lang="ts">
  import "./LabeledInput.scss";

  type Props = {
    label: string;
    value: string;
    testId: string;
    type?: "text" | "email" | "password";
    placeholder?: string;
  };

  let {
    label,
    value = $bindable(),
    testId,
    type = "text",
    placeholder = "",
  }: Props = $props();
</script>

<label class="labeled-input">
  <span>{label}</span>
  <input
    class="input"
    {type}
    {placeholder}
    data-testid={testId}
    bind:value
  />
</label>
