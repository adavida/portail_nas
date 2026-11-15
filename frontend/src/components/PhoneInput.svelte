<script lang="ts">
  import {
    AsYouType,
    getCountries,
    getCountryCallingCode,
    isSupportedCountry,
    parsePhoneNumberFromString,
    type CountryCode,
  } from "libphonenumber-js/min";
  import "./PhoneInput.scss";

  type Props = {
    label: string;
    value: string;
    testId: string;
    defaultCountry?: string;
  };

  function normalize(raw: string | undefined): CountryCode {
    return raw && isSupportedCountry(raw) ? (raw as CountryCode) : "FR";
  }

  function digitsOf(raw: string): string {
    return raw.replace(/\D/g, "");
  }

  function flagOf(countryCode: string): string {
    return String.fromCodePoint(
      ...[...countryCode].map((letter) => 127397 + letter.charCodeAt(0)),
    );
  }

  const regionNames = new Intl.DisplayNames(["en"], { type: "region" });

  function nameOf(countryCode: CountryCode): string {
    return regionNames.of(countryCode) ?? countryCode;
  }

  let {
    label,
    value = $bindable(),
    testId,
    defaultCountry = "FR",
  }: Props = $props();

  const countries = getCountries().map((code) => ({
    code,
    dial: String(getCountryCallingCode(code)),
    text: `${flagOf(code)} +${getCountryCallingCode(code)} ${nameOf(code)}`,
  }));

  // shortcut: shared dial prefixes (+1, +7) only switch once unambiguous — the picker covers them
  function countryByDial(digits: string): CountryCode | null {
    const exact = countries.filter((option) => option.dial === digits);
    const ambiguous = countries.filter(
      (option) => option.dial.startsWith(digits) && option.dial !== digits,
    );
    return exact.length === 1 && ambiguous.length === 0
      ? exact[0].code
      : null;
  }

  function nationalOf(raw: string, region: CountryCode): string {
    const dial = `+${getCountryCallingCode(region)}`;
    const candidate = raw.startsWith("+")
      ? raw
      : `+${dial.slice(1)}${digitsOf(raw).replace(/^0/, "")}`;
    const parsed = parsePhoneNumberFromString(candidate, region);
    const formatted = parsed
      ? parsed.formatInternational()
      : new AsYouType(region).input(candidate);
    return formatted.startsWith(dial)
      ? formatted.slice(dial.length).trimStart()
      : formatted.trim();
  }

  // svelte-ignore state_referenced_locally
  const startCountry = normalize(defaultCountry);
  let country = $state<CountryCode | null>(
    parsePhoneNumberFromString(value, startCountry)?.country ?? startCountry,
  );
  let display = $state(nationalOf(value, startCountry));
  let emitted = $state(value);
  let touched = $state(false);
  let open = $state(false);
  let query = $state("");

  const filtered = $derived(
    countries.filter((option) =>
      option.text.toLowerCase().includes(query.toLowerCase()),
    ),
  );

  function apply(raw: string, region: CountryCode | null): void {
    if (region) {
      const dial = `+${getCountryCallingCode(region)}`;
      const digits = digitsOf(raw).replace(/^0/, "");
      const candidate = raw.startsWith("+") ? raw : `${dial}${digits}`;
      const parsed = parsePhoneNumberFromString(candidate, region);
      const effective = parsed?.country ?? region;
      country = effective;
      display = nationalOf(raw, effective);
      value = parsed
        ? parsed.number
        : digits || raw.startsWith("+")
          ? candidate
          : "";
      emitted = value;
      return;
    }
    const all = digitsOf(raw);
    if (!all) {
      display = "";
      value = "";
      emitted = "";
      return;
    }
    const byPrefix = countryByDial(all);
    if (byPrefix) {
      apply(all.slice(getCountryCallingCode(byPrefix).length), byPrefix);
      return;
    }
    const parsed = parsePhoneNumberFromString(`+${all}`);
    if (parsed?.country) {
      apply(digitsOf(raw).slice(getCountryCallingCode(parsed.country).length), parsed.country);
      return;
    }
    display = new AsYouType().input(`+${all}`);
    value = `+${all}`;
    emitted = value;
  }

  $effect(() => {
    if (value !== emitted) {
      apply(value, country);
      return;
    }
    if (value === "" && !touched) {
      const next = normalize(defaultCountry);
      if (next !== country) {
        country = next;
        display = "";
      }
    }
  });

  function onInput(event: Event): void {
    const target = event.target as HTMLInputElement;
    touched = true;
    apply(target.value, country);
    target.value = display;
  }

  function onKeydown(event: KeyboardEvent): void {
    if (event.key === "Backspace" && display === "" && country !== null) {
      event.preventDefault();
      country = null;
      touched = true;
      value = "";
      emitted = "";
    }
  }

  function pick(picked: CountryCode): void {
    open = false;
    query = "";
    touched = true;
    apply(display, picked);
  }
</script>

<label class="phone-input">
  <span>{label}</span>
  <div class="phone-field">
    <button
      type="button"
      class="phone-trigger"
      data-testid={`${testId}-country`}
      onclick={(event) => {
        event.preventDefault();
        event.stopPropagation();
        open = !open;
      }}
    >
      {country ? flagOf(country) : "🌐"} ▾
    </button>
    {#if country}
      <span class="phone-dial" data-testid={`${testId}-dial`}
        >+{getCountryCallingCode(country)}</span
      >
    {/if}
    <input
      class="phone-number"
      type="tel"
      data-testid={testId}
      value={display}
      oninput={onInput}
      onkeydown={onKeydown}
    />
    {#if open}
      <div
        class="phone-backdrop"
        role="presentation"
        aria-hidden="true"
        onclick={() => (open = false)}
      ></div>
      <div class="phone-list" data-testid={`${testId}-list`}>
        <input
          class="phone-search"
          type="text"
          data-testid={`${testId}-search`}
          placeholder="+32 Belgium"
          bind:value={query}
        />
        <ul>
          {#each filtered as option (option.code)}
            <li>
              <button type="button" onclick={() => pick(option.code)}>
                {option.text}
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  </div>
</label>
