<script lang="ts">
  import Button from '../../ui/Button.svelte';
  import InlineConfirm from '../../ui/InlineConfirm.svelte';
  import SettingRow from '../../ui/SettingRow.svelte';
  import SettingRows from '../../ui/SettingRows.svelte';
  import StatusDot from '../../ui/StatusDot.svelte';
  import Tag from '../../ui/Tag.svelte';
  import TextField from '../../ui/TextField.svelte';
  import { onMount } from 'svelte';
  import { apiInfo, regenerateApiToken, type ApiInfo } from '../../api';
  import { lib, openLink, report } from '../library/libraryState.svelte';
  import { LATEST_RELEASE_URL } from './settingsLogic';
  import { t } from '../../strings';

  // Settings mounts this again on a profile switch: the token, a question or a notice belong to one profile.
  const profile = lib.profileId;
  let info = $state<ApiInfo | null>(null);
  let confirming = $state(false);
  let notice = $state<string | null>(null);
  const uid = $props.id();

  onMount(() => {
    apiInfo(profile)
      .then((loaded) => (info = loaded))
      .catch(report);
  });

  async function copy(token: string): Promise<void> {
    notice = null;
    try {
      await navigator.clipboard.writeText(token);
      notice = t.tokenCopied;
    } catch (err) {
      report(err);
    }
  }

  async function regenerate(current: ApiInfo): Promise<void> {
    confirming = false;
    notice = null;
    try {
      const token = await regenerateApiToken(profile);
      info = { ...current, token };
      notice = t.tokenRegenerated;
    } catch (err) {
      report(err);
    }
  }
</script>

{#if info}
  <SettingRows>
    <SettingRow label={info.listening ? t.apiListening(info.port) : t.apiPortBusy(info.port)}>
      {#snippet lead()}<StatusDot status={info?.listening ? 'ready' : 'failed'} />{/snippet}
    </SettingRow>
    <SettingRow label={t.tokenLabel} for="{uid}-token">
      <div class="token"><TextField id="{uid}-token" mono readonly value={info.token} /></div>
    </SettingRow>
    <SettingRow
      help={confirming ? t.confirmRegenerateToken : notice}
      helpRole={confirming ? 'alert' : 'status'}
    >
      {#if confirming}
        <InlineConfirm
          label={t.confirmRegenerateToken}
          yes={t.confirmRegenerateTokenYes}
          cancel={t.cancel}
          onyes={() => info && regenerate(info)}
          oncancel={() => (confirming = false)}
        />
      {:else}
        <Button onclick={() => info && copy(info.token)}>{t.copyToken}</Button>
        <Button onclick={() => (confirming = true)}>{t.regenerateToken}</Button>
      {/if}
    </SettingRow>
  </SettingRows>
  <div class="steps">
    <section>
      <h2 class="section-label">
        {t.extensionChromiumTitle}
        <Tag>{t.extensionChromiumRecommended}</Tag>
      </h2>
      <ol>
        {#each t.extensionChromiumSteps as step (step)}<li>{step}</li>{/each}
      </ol>
      <div class="download">
        <Button onclick={() => openLink(LATEST_RELEASE_URL)}>{t.extensionChromiumDownload}</Button>
      </div>
    </section>
    <section>
      <h2 class="section-label">
        {t.extensionSafariTitle}
        <Tag>{t.extensionSafariExperimental}</Tag>
      </h2>
      <ol>
        {#each t.extensionSafariSteps as step (step)}<li>{step}</li>{/each}
      </ol>
      <p>{t.extensionSafariNote}</p>
    </section>
  </div>
{/if}

<style>
  .token {
    flex: 2;
    min-width: 0;
  }

  .steps {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-xl);
    margin-top: var(--space-lg);
    color: var(--muted);
    font: var(--type-help);
  }

  h2 {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-top: 0;
  }

  ol,
  p {
    margin: 0;
    padding-left: var(--space-lg);
  }

  p,
  .download {
    margin-top: var(--space-md);
  }

  .download {
    padding-left: var(--space-lg);
  }
</style>
