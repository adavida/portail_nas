<script lang="ts">
  import { onMount } from "svelte";
  import { whoami, type AttrEntry } from "../api/auth";
  import {
    updateMe,
    updateMyPassword,
    passwordPolicy,
  } from "../api/me";
  import { lang, t, translate } from "../i18n";
  import { toast } from "../components/toast";
  import LabeledInput from "../components/LabeledInput.svelte";
  import PhoneInput from "../components/PhoneInput.svelte";
  import Tabs from "../components/Tabs.svelte";
  import "./Profile.scss";

  let uid = $state("");
  let groups = $state<string[]>([]);
  let name = $state("");
  let email = $state("");
  let savedName = $state("");
  let savedEmail = $state("");
  let password = $state("");
  let confirmPassword = $state("");
  let minLength = $state(0);
  let defaultCountry = $state("FR");
  let loading = $state(true);
  let tab = $state("infos");
  let attrs = $state<AttrEntry[]>([]);
  let attrValues = $state<Record<string, string>>({});
  let savedAttrValues = $state<Record<string, string>>({});

  const serializedAttrs = (values: Record<string, string>) =>
    JSON.stringify(Object.entries(values).sort());
  const dirty = $derived(
    name !== savedName ||
      email !== savedEmail ||
      serializedAttrs(attrValues) !== serializedAttrs(savedAttrValues),
  );
  const passwordOk = $derived(
    !!password && (minLength === 0 || password.length >= minLength),
  );
  const passwordsMatch = $derived(passwordOk && password === confirmPassword);

  onMount(async () => {
    passwordPolicy()
      .then((policy) => {
        minLength = policy.minLength;
        defaultCountry = policy.defaultCountry;
      })
      .catch(() => {});
    const me = await whoami();
    if (me !== "relogin") {
      uid = me.preferred_username ?? me.sub;
      groups = me.groups;
      name = savedName = me.name ?? "";
      email = savedEmail = me.email ?? "";
      attrs = me.attrs ?? [];
      const initial = Object.fromEntries(
        attrs.map((entry) => [entry.label, entry.value]),
      );
      attrValues = { ...initial };
      savedAttrValues = { ...initial };
    }
    loading = false;
  });

  async function save() {
    const payload = attrs.map((entry) => ({
      label: entry.label,
      value: attrValues[entry.label] ?? "",
    }));
    let result;
    try {
      result = await updateMe(name, email, payload);
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    if (result.profile_saved) {
      savedName = name;
      savedEmail = email;
    }
    if (result.attrs_saved) {
      savedAttrValues = { ...attrValues };
    }
    if (result.error) {
      toast(result.error, true);
      return;
    }
    toast(translate("common.saved"));
  }

  async function changePassword() {
    try {
      await updateMyPassword(password);
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    password = "";
    confirmPassword = "";
    toast(translate("common.password_changed"));
  }
</script>

<section>
  <h2>{$t("nav.profile")}</h2>
  {#if loading}
    <p class="muted" data-testid="profile-loading">{$t("common.loading")}</p>
  {:else}
    <div class="card profile-card" data-testid="profile-form">
      <div class="profile-id">
        <span class="profile-avatar" aria-hidden="true">
          {(name || uid).slice(0, 1).toUpperCase()}
        </span>
        <div>
          <span class="profile-uid">{uid}</span>
          {#if groups.length}
            <div class="profile-groups">
              {#each groups as group (group)}
                <span class="profile-tag">{group}</span>
              {/each}
            </div>
          {/if}
        </div>
      </div>

      <Tabs
        tabs={[
          {
            id: "infos",
            label: $t("profile.informations"),
            testId: "profile-tab-infos",
          },
          {
            id: "password",
            label: $t("field.password"),
            testId: "profile-tab-password",
          },
        ]}
        bind:active={tab}
      />

      {#if tab === "infos"}
        <div class="profile-fields">
          <LabeledInput
            label={$t("field.name")}
            testId="profile-name"
            bind:value={name}
          />
          <LabeledInput
            label={$t("field.email")}
            type="email"
            testId="profile-email"
            bind:value={email}
          />
        </div>
        {#if attrs.length}
          <h3 class="profile-subtitle">{$t("profile.extra_informations")}</h3>
          <div class="profile-fields" data-testid="profile-attrs">
            {#each attrs as entry (entry.label)}
              {#if entry.type === "telephone"}
                <PhoneInput
                  label={entry.names?.[$lang] ?? entry.label}
                  testId={`profile-attr-${entry.label}`}
                  defaultCountry={defaultCountry}
                  bind:value={attrValues[entry.label]}
                />
              {:else}
                <LabeledInput
                  label={entry.names?.[$lang] ?? entry.label}
                  type={entry.type}
                  testId={`profile-attr-${entry.label}`}
                  maxLength={entry.max_chars}
                  min={entry.min}
                  max={entry.max}
                  bind:value={attrValues[entry.label]}
                />
              {/if}
            {/each}
          </div>
        {/if}
        <div class="row-actions profile-save">
          <button
            class="btn btn-primary"
            type="button"
            data-testid="profile-save"
            disabled={!dirty}
            onclick={() => void save()}
          >
            {$t("common.validate")}
          </button>
        </div>
      {:else}
        <ul class="profile-conditions" data-testid="profile-password-hint">
          {#if minLength}
            <li class:ok={passwordOk}>
              {$t("profile.password_rules", { min: String(minLength) })}
            </li>
          {/if}
        </ul>
        <div class="profile-fields">
          <LabeledInput
            label={$t("profile.new_password")}
            type="password"
            testId="profile-password"
            bind:value={password}
          />
          <LabeledInput
            label={$t("profile.confirm_password")}
            type="password"
            testId="profile-password-confirm"
            bind:value={confirmPassword}
          />
        </div>
        <div class="row-actions profile-save">
          <button
            class="btn btn-primary"
            type="button"
            data-testid="profile-password-save"
            disabled={!passwordsMatch}
            onclick={() => void changePassword()}
          >
            {$t("common.edit")}
          </button>
        </div>
      {/if}
    </div>
  {/if}
</section>
