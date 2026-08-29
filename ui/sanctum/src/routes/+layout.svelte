<script lang="ts">
  import '@metocast/design-system/tokens.css';
  import '@metocast/design-system/fonts.css';
  import '@metocast/design-system/base.css';
  import '$lib/i18n';
  import { onMount } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { enableWindowGlass, connectWs, getAppMode, onConnectLink } from '@metocast/core-client';
  import type { WsStatus, HostMode, ConnectLink } from '@metocast/core-client';
  import {
    ToastOverlay,
    Toast,
    Glyph,
    NotificationCentre,
    Dialog,
    Button,
  } from '@metocast/design-system';
  import { initCore } from '$lib/core';
  import { initScheme } from '$lib/scheme.svelte';
  import { handleWs, registerLive } from '$lib/live.svelte';
  import {
    railItems,
    dismissRail,
    allNotifications,
    unreadCount,
    isCentreOpen,
    closeCentre,
    clearAll,
    type Notification,
  } from '$lib/notifications.svelte';
  import Nav from '$lib/Nav.svelte';
  import Setup from '$lib/Setup.svelte';

  let { children } = $props();

  let status = $state<WsStatus>('connecting');
  let phase = $state<'loading' | 'setup' | 'app'>('loading');
  let pendingLink = $state<ConnectLink | null>(null);
  let overrideLink = $state<ConnectLink | null>(null);

  function receiveConnectLink(link: ConnectLink) {
    if (phase === 'setup') pendingLink = link;
    else overrideLink = link;
  }

  function acceptOverride() {
    pendingLink = overrideLink;
    overrideLink = null;
    phase = 'setup';
  }

  onMount(async () => {
    initScheme();
    enableWindowGlass()
      .then(() => document.body.classList.add('glass'))
      .catch(() => {});

    let mode: HostMode | null = 'server';
    try {
      mode = await getAppMode();
    } catch {
      mode = 'server';
    }
    if (mode !== 'server' && mode !== 'client') {
      phase = 'setup';
    } else {
      try {
        await initCore();
      } catch {
        /* no desktop host / not configured — stay on defaults */
      }
      connectWs({
        onStatus: (s) => (status = s),
        onOpen: () => registerLive(),
        onMessage: (msg) => handleWs(msg),
      });
      phase = 'app';
    }
    onConnectLink(receiveConnectLink);
  });
</script>

<div class="titlebar" data-tauri-drag-region aria-hidden="true"></div>

{#if phase === 'setup'}
  <Setup connect={pendingLink} />
{:else if phase === 'app'}
  {#snippet connectorGlyph(t: Notification)}
    {#if t.brand}
      <Glyph size={34} label={t.source}>
        {#snippet mark()}<svg
            viewBox="0 0 24 24"
            width="19"
            height="19"
            fill="currentColor"
            aria-hidden="true"><path d={t.brand} /></svg
          >{/snippet}
      </Glyph>
    {:else}
      <Glyph char={t.source.charAt(0)} size={34} />
    {/if}
  {/snippet}

  <Nav />

  {#if status !== 'connected'}
    <div class="conn" role="status">
      {status === 'connecting' ? $_('connection.connecting') : $_('connection.lost')}
    </div>
  {/if}

  <main class="content">
    {@render children()}
  </main>

  <div class="toast-host">
    <ToastOverlay label={$_('toast.region')} priority="assertive">
      {#each railItems() as t (t.id)}
        <Toast
          kind={t.kind}
          source={t.source}
          title={t.title}
          body={t.body}
          tone={t.tier}
          state={t.state}
          actions={(t.actions ?? []).map((a) => ({
            label: a.label,
            primary: a.primary,
            onclick: a.run,
          }))}
          remediation={t.remediation}
          whyLabel={$_('notif.showSteps')}
          hideWhyLabel={$_('notif.hideSteps')}
          dismissLabel={$_('toast.dismiss')}
          ondismiss={() => dismissRail(t.id)}
        >
          {#snippet mark()}{@render connectorGlyph(t)}{/snippet}
        </Toast>
      {/each}
    </ToastOverlay>
  </div>

  <NotificationCentre
    open={isCentreOpen()}
    eyebrow={$_('notif.eyebrow')}
    title={unreadCount() > 0 || allNotifications().length
      ? $_('notif.count', { values: { n: allNotifications().length } })
      : $_('notif.allClear')}
    empty={allNotifications().length === 0}
    emptyTitle={$_('notif.emptyTitle')}
    emptyHint={$_('notif.emptyHint')}
    clearLabel={$_('notif.clearAll')}
    onclear={clearAll}
    onclose={closeCentre}
  >
    {#each allNotifications() as t (t.id)}
      <Toast
        kind={t.kind}
        source={t.source}
        title={t.title}
        body={t.body}
        tone={t.tier}
        state={t.state}
        actions={(t.actions ?? []).map((a) => ({
          label: a.label,
          primary: a.primary,
          onclick: a.run,
        }))}
        remediation={t.remediation}
        whyLabel={$_('notif.showSteps')}
        hideWhyLabel={$_('notif.hideSteps')}
        dismissLabel={$_('toast.dismiss')}
        ondismiss={() => dismissRail(t.id)}
      >
        {#snippet mark()}{@render connectorGlyph(t)}{/snippet}
        {#snippet detail()}
          {#if t.group?.length}
            <ul class="group">
              {#each t.group as g (g.source)}<li>{g.source} — {g.label}</li>{/each}
            </ul>
          {/if}
        {/snippet}
      </Toast>
    {/each}
  </NotificationCentre>
{/if}

<Dialog
  open={overrideLink !== null}
  eyebrow={$_('setup.override.eyebrow')}
  title={$_('setup.override.title')}
  onclose={() => (overrideLink = null)}
>
  {$_('setup.override.body', { values: { url: overrideLink?.url ?? '' } })}
  {#snippet footer()}
    <Button variant="secondary" compact onclick={() => (overrideLink = null)}>
      {$_('setup.override.keep')}
    </Button>
    <Button variant="primary" compact onclick={acceptOverride}
      >{$_('setup.override.replace')}</Button
    >
  {/snippet}
</Dialog>

<div id="overlays"></div>

<style>
  .titlebar {
    position: fixed;
    inset: 0 0 auto 0;
    height: max(44px, env(safe-area-inset-top, 44px));
    z-index: 100;
  }

  :global(body.glass) {
    background: color-mix(in srgb, var(--surface-outside) 82%, transparent);
  }

  /* A quiet indicator in the top-right, not a full-width bar. */
  .conn {
    position: fixed;
    top: 52px;
    right: 16px;
    z-index: 60;
    padding: 4px 10px;
    font-family: var(--font-label);
    font-size: 10px;
    letter-spacing: 0.6px;
    background: var(--surface-raised);
    color: var(--text-muted);
    border: 1px solid color-mix(in srgb, var(--text-primary) 14%, transparent);
  }

  /* Not a fixed scroll pane: on iOS that becomes the containing block for the
     fixed children inside it, clipping every page-level Sheet at the nav. */
  .content {
    padding-top: max(44px, env(safe-area-inset-top, 44px));
    padding-bottom: calc(57px + max(14px, env(safe-area-inset-bottom, 14px)));
  }

  .group {
    margin: 8px 0 0;
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.5;
    list-style: none;
    padding-left: 0;
  }

  /* On tablet/desktop the toast rail settles to the right at a capped width, rather
     than stretching the full window as the DS overlay does by default. */
  @media (min-width: 760px) {
    .toast-host :global(.overlay) {
      left: auto;
      max-width: 380px;
    }
  }

  @media (min-width: 980px) {
    .content {
      padding-top: 44px;
      padding-bottom: 0;
      margin-left: 226px;
    }
  }
</style>
