<script lang="ts">
  import { onMount } from "svelte";
  import { whoami } from "../api/auth";
  import { updateMe, updateMyPassword, passwordPolicy } from "../api/me";
  import { t, translate } from "../i18n";
  import { toast } from "../components/toast";
  import LabeledInput from "../components/LabeledInput.svelte";
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
  let loading = $state(true);
  let tab = $state("infos");

  const dirty = $derived(name !== savedName || email !== savedEmail);
  const passwordOk = $derived(
    !!password && (minLength === 0 || password.length >= minLength),
  );
  const passwordsMatch = $derived(passwordOk && password === confirmPassword);

  onMount(async () => {
    passwordPolicy()
      .then((n) => (minLength = n))
      .catch(() => {});
    const me = await whoami();
    if (me !== "relogin") {
      uid = me.preferred_username ?? me.sub;
      groups = me.groups;
      name = savedName = me.name ?? "";
      email = savedEmail = me.email ?? "";
    }
    loading = false;
  });

  async function save() {
    try {
      await updateMe(name, email);
    } catch (e) {
      toast((e as Error).message, true);
      return;
    }
    savedName = name;
    savedEmail = email;
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
