<script lang="ts">
  import { onMount } from 'svelte';
  import { MediaQuery } from 'svelte/reactivity';
  import { _, locale } from 'svelte-i18n';
  import {
    PageHeader,
    Dot,
    Segmented,
    TransportDock,
    SlideSearch,
    SlideQueue,
    SectionLabel,
    List,
    Row,
    EmptyState,
    IconButton,
    Sheet,
    LabelledInput,
  } from '@metocast/design-system';
  import type { TransportAction, SlideResult, QueueSlot } from '@metocast/design-system';
  import {
    sendWsCommand,
    searchFiles,
    openFile,
    listFolders,
    addFolder,
    removeFolder,
    keynoteCloseAll,
    pickDirectory,
    hostCapabilities,
  } from '@metocast/core-client';
  import type { PptFile, PptFolder } from '@metocast/core-client/schemas/ws-messages';
  import { live } from '$lib/live.svelte';
  import { presenterUrl, appMode } from '$lib/core';
  import { relAge } from '$lib/format';
  import NotifBell from '$lib/NotifBell.svelte';
  import SwipeReveal from '$lib/SwipeReveal.svelte';

  const mobile = new MediaQuery('(pointer: coarse)');
  const loc = $derived($locale ?? 'en');
  const mode = $derived(live.useWebPresenter ? 'web' : 'keynote');

  const web = $derived(live.presenter);
  const kn = $derived(live.keynote);
  const loaded = $derived(mode === 'web' ? !!web?.loaded : !!(kn?.slideshowActive || kn?.documentName));
  const deckName = $derived(
    mode === 'web'
      ? (web?.filePath?.split('/').pop() ?? '')
      : (kn?.documentName ?? ''),
  );
  const current = $derived(mode === 'web' ? (web?.currentSlide ?? 0) : (kn?.currentSlide ?? 0));
  const total = $derived(mode === 'web' ? (web?.totalSlides ?? 0) : (kn?.totalSlides ?? 0));
  const currentSlide = $derived(
    mode === 'web' && loaded ? web?.slides?.find((s) => s.index === web.currentSlide) : undefined,
  );

  function setMode(m: string) {
    sendWsCommand('presentation.set_use_web_presenter', { enabled: m === 'web' });
  }

  function tx(cmd: 'first' | 'prev' | 'next' | 'last' | 'start') {
    sendWsCommand(`presentation.${cmd}`);
  }
  function stop() {
    if (!sendWsCommand('keynote.close_all')) keynoteCloseAll();
  }

  const actions = $derived<TransportAction[]>([
    { icon: 'first', label: $_('presentations.first'), disabled: !loaded, onclick: () => tx('first') },
    { icon: 'prev', label: $_('presentations.prev'), disabled: !loaded || current <= 1, onclick: () => tx('prev') },
    { icon: 'next', label: $_('presentations.next'), variant: 'primary', disabled: !loaded || current >= total, onclick: () => tx('next') },
    { icon: 'last', label: $_('presentations.last'), disabled: !loaded || current >= total, onclick: () => tx('last') },
    ...(mode === 'keynote'
      ? [
          { icon: 'play' as const, label: $_('presentations.start'), disabled: !loaded, onclick: () => tx('start') },
          { icon: 'stop' as const, label: $_('presentations.stop'), variant: 'stop' as const, disabled: !loaded, onclick: stop },
        ]
      : [{ icon: 'stop' as const, label: $_('presentations.unload'), variant: 'stop' as const, disabled: !loaded, onclick: stop }]),
  ]);

  const status = $derived(loaded ? $_(`presentations.mode.${mode}`) + ' · ' + $_('presentations.presenting') : $_('presentations.standby'));

  // ── Deck search ───────────────────────────────────────────────────────────────
  let searchOpen = $state(false);
  let filter = $state('');
  let timer: ReturnType<typeof setTimeout>;

  function runSearch() {
    clearTimeout(timer);
    timer = setTimeout(() => {
      if (!sendWsCommand('ppt.search', { filter })) searchFiles(filter);
    }, 250);
  }
  function openSearch() {
    searchOpen = !searchOpen;
    if (searchOpen) runSearch();
  }
  $effect(() => {
    filter;
    if (searchOpen || filter.length > 0) runSearch();
  });

  const results = $derived<SlideResult[]>(
    searchOpen || filter.length > 0
      ? live.pptResults.slice(0, 8).map((f) => ({ id: f.id, group: folderName(f.folderId), title: f.name }))
      : [],
  );

  // ── Preload queue ─────────────────────────────────────────────────────────────
  let slotFiles = $state<(PptFile | null)[]>([null, null, null, null, null]);
  const queueFull = $derived(slotFiles.every(Boolean));
  const slots = $derived<QueueSlot[]>(
    slotFiles.map((f, i) => ({ index: i, title: f?.name, loaded: !!f && web?.filePath === f.path })),
  );

  function fileById(id: string): PptFile | undefined {
    return live.pptResults.find((f) => f.id === id);
  }
  function queueResult(r: SlideResult) {
    const f = fileById(r.id);
    if (!f) return;
    const i = slotFiles.findIndex((s) => !s);
    if (i < 0) return;
    slotFiles[i] = f;
  }
  function openPath(path: string) {
    if (!sendWsCommand('presentation.open', { file_path: path })) openFile(path);
  }
  function openResult(r: SlideResult) {
    const f = fileById(r.id);
    if (f) openPath(f.path);
  }

  // ── Settings sheet ────────────────────────────────────────────────────────────
  let settingsOpen = $state(false);

  let url = $state('');
  let copied = $state(false);
  const clients = $derived(
    live.clients.map((c) => ({
      id: c.id,
      name: c.label,
      address: c.hostname ?? c.userAgent ?? c.id.slice(0, 8),
      detail: relAge(c.connectedAt, loc) + (c.latencyMs != null ? ` · ${c.latencyMs}ms` : ''),
    })),
  );

  async function copyUrl() {
    try {
      await navigator.clipboard?.writeText(url);
      copied = true;
      setTimeout(() => (copied = false), 1300);
    } catch {
      /* clipboard blocked */
    }
  }

  let folders = $state<PptFolder[]>([]);
  function folderName(id: string): string {
    return folders.find((f) => f.id === id)?.name ?? '';
  }
  async function loadFolders() {
    try {
      folders = await listFolders();
    } catch {
      /* offline */
    }
  }
  // Only the desktop shell that *is* the core sees the same filesystem the core reads,
  // so it gets a native picker; every other window types the path and the core validates it.
  const canPick = $derived(hostCapabilities.dialogs && appMode() === 'server');
  let newPath = $state('');
  let folderError = $state('');
  let adding = $state(false);

  async function submitFolder(path: string) {
    folderError = '';
    adding = true;
    try {
      await addFolder(path, path.split(/[/\\]/).filter(Boolean).pop() || path);
      newPath = '';
      await loadFolders();
      if (searchOpen) runSearch();
    } catch (e) {
      folderError = e instanceof Error ? e.message : String(e);
    } finally {
      adding = false;
    }
  }

  async function addFolderFlow() {
    const path = await pickDirectory();
    if (path) await submitFolder(path);
  }
  async function dropFolder(id: string) {
    await removeFolder(id);
    await loadFolders();
  }

  onMount(async () => {
    url = await presenterUrl();
    sendWsCommand('clients.list');
    await loadFolders();
  });
</script>

<PageHeader eyebrow={$_(`presentations.mode.${mode}`)} title={$_('presentations.title')}>
  {#snippet trailing()}
    {#if loaded}<Dot color="var(--status-ok)" size={6} pulse />{/if}
    <NotifBell />
    <IconButton
      icon="gear"
      label={$_('screens.settings.title')}
      variant="circle"
      onclick={() => (settingsOpen = true)}
    />
  {/snippet}
</PageHeader>

<div class="workspace">
  <div class="remote-col">
    <TransportDock
      label={$_('presentations.transport')}
      {status}
      current={loaded ? deckName : $_('presentations.noDeck')}
      position={loaded ? `${current} / ${total}` : '—'}
      {actions}
    />

    <SlideSearch
      results={results}
      label={$_('presentations.search')}
      searchLabel={$_('presentations.search')}
      placeholder={mobile.current ? $_('presentations.tapToSearch') : $_('presentations.searchPlaceholder')}
      bind:filter
      emptyMessage={searchOpen || filter.length > 0 ? $_('presentations.noMatch') : $_('presentations.tapToSearch')}
      openLabel={$_('presentations.open')}
      queueLabel={$_('presentations.queue')}
      queueDisabled={queueFull}
      numpad={mobile.current}
      ontrigger={openSearch}
      onopen={openResult}
      onqueue={queueResult}
    />
  </div>

  <aside class="secondary-col">
    <SlideQueue
      {slots}
      label={$_('presentations.queueLabel')}
      summary={$_('presentations.queueSummary', { values: { n: slotFiles.filter(Boolean).length, total: slotFiles.length } })}
      openLabel={$_('presentations.open')}
      clearLabel={$_('presentations.clear')}
      emptyLabel={$_('presentations.emptySlot')}
      onopen={(s) => { const f = slotFiles[s.index]; if (f) openPath(f.path); }}
      onclear={(s) => (slotFiles[s.index] = null)}
    />

    <SectionLabel>{$_('presentations.preview')}</SectionLabel>
    <div class="preview">
      {#if currentSlide}
        <span class="pos">{current} / {total}</span>
        <div class="slide">
          {#each currentSlide.paragraphs as p, i (i)}
            {#each p.lines as line, j (j)}<p>{line}</p>{/each}
          {/each}
        </div>
      {:else}
        <EmptyState title={$_('presentations.previewWaiting')} hint={$_('presentations.previewWaitingHint')} />
      {/if}
    </div>
  </aside>
</div>

<Sheet
  bind:open={settingsOpen}
  title={$_('screens.settings.title')}
  eyebrow={$_('presentations.title')}
>
  <SectionLabel>{$_('presentations.modeLabel')}</SectionLabel>
  <div class="mode-seg">
    <Segmented
      label={$_('presentations.modeLabel')}
      value={mode}
      options={[
        { value: 'web', label: $_('presentations.mode.web') },
        { value: 'keynote', label: $_('presentations.mode.keynote') },
      ]}
      onchange={setMode}
    />
  </div>

  {#if mode === 'web'}
    <SectionLabel>{$_('presentations.presenterSupport')}</SectionLabel>
    <div class="url-bar">
      <span>{url}</span>
      <button type="button" onclick={copyUrl}>
        {copied ? $_('presentations.copied') : $_('presentations.copy')}
      </button>
    </div>

    <SectionLabel hint={$_('presentations.clientsSummary', { values: { n: clients.length } })}>
      {$_('presentations.clients')}
    </SectionLabel>
    <List>
      {#if clients.length === 0}
        <div class="no-clients">{$_('presentations.noClients')}</div>
      {:else}
        {#each clients as c, i (c.id)}
          <Row title={c.name} meta={c.address} detail={c.detail} chevron={false} last={i === clients.length - 1} />
        {/each}
      {/if}
    </List>
  {/if}

  <div class="folders-bar">
    <span class="folders-count">{$_('presentations.foldersHint', { values: { n: folders.length } })}</span>
    {#if canPick}
      <button class="add-folder-btn" type="button" disabled={adding} onclick={addFolderFlow}>
        + {$_('presentations.addFolder')}
      </button>
    {/if}
  </div>
  {#if canPick}
    {#if folderError}<p class="folder-error">{folderError}</p>{/if}
  {:else}
    <div class="folder-add">
      <LabelledInput
        label={$_('presentations.folderPath')}
        placeholder={$_('presentations.folderPathPlaceholder')}
        bind:value={newPath}
        error={folderError}
      />
      <button
        class="add-folder-btn"
        type="button"
        disabled={adding || newPath.trim().length === 0}
        onclick={() => submitFolder(newPath.trim())}
      >
        + {$_('presentations.addFolder')}
      </button>
    </div>
  {/if}
  <List>
    {#each folders as f, i (f.id)}
      <SwipeReveal onCommit={() => dropFolder(f.id)} commitLabel={$_('presentations.removeFolder')}>
        <div class="folder-item" class:folder-last={i === folders.length - 1}>
          <strong class="folder-name">{f.name}</strong>
          <span class="folder-path">{f.path}</span>
        </div>
      </SwipeReveal>
    {/each}
  </List>
</Sheet>

<style>
  .preview {
    margin: 0 24px 8px;
  }
  .preview .pos {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 1.4px;
    color: var(--text-muted);
  }
  .slide {
    border: 1px solid color-mix(in srgb, var(--text-primary) 14%, transparent);
    padding: 16px;
    margin-top: 6px;
    text-align: center;
  }
  .slide p {
    margin: 2px 0;
    font-family: var(--font-display);
    font-size: 16px;
    color: var(--text-primary);
  }
  .url-bar {
    display: flex;
    margin: 0 24px 8px;
    border: 1px solid color-mix(in srgb, var(--text-primary) 14%, transparent);
    background: var(--surface-raised);
  }
  .url-bar span {
    flex: 1;
    min-width: 0;
    padding: 12px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-primary);
    word-break: break-all;
  }
  .url-bar button {
    padding: 0 16px;
    border: 0;
    background: var(--surface-inverse);
    color: var(--text-inverse);
    cursor: pointer;
    font-family: var(--font-label);
    font-size: 10px;
    letter-spacing: 1.4px;
    text-transform: uppercase;
    white-space: nowrap;
    min-height: var(--ui-target-min);
  }
  .no-clients {
    padding: 12px 24px;
    font-family: var(--font-display);
    font-style: italic;
    font-size: 14px;
    color: var(--text-muted);
  }
  .mode-seg {
    padding: 0 24px 16px;
  }
  .folders-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 24px;
    border-top: 1px solid color-mix(in srgb, var(--text-primary) 10%, transparent);
    border-bottom: 1px solid color-mix(in srgb, var(--text-primary) 10%, transparent);
  }
  .folders-count {
    font-family: var(--font-label);
    font-size: 10px;
    letter-spacing: 1.4px;
    text-transform: uppercase;
    color: var(--text-muted);
  }
  .add-folder-btn {
    background: var(--surface-inverse);
    color: var(--text-inverse);
    border: 0;
    padding: 0 14px;
    min-height: var(--ui-target-min);
    font-family: var(--font-label);
    font-size: 10px;
    letter-spacing: 1.4px;
    text-transform: uppercase;
    cursor: pointer;
  }
  .add-folder-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .folder-add {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 24px;
  }
  .folder-add :global(.chamber) {
    flex: 1;
    min-width: 0;
  }
  .folder-error {
    margin: 0;
    padding: 8px 24px;
    font-size: 12px;
    color: var(--status-error);
  }
  .folder-item {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 16px 24px;
    border-bottom: 1px solid color-mix(in srgb, var(--text-primary) 10%, transparent);
  }
  .folder-last {
    border-bottom: 0;
  }
  .folder-name {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    flex-shrink: 0;
  }
  .folder-path {
    flex: 1;
    min-width: 0;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .workspace,
  .remote-col,
  .secondary-col {
    display: contents;
  }

  @media (min-width: 760px) {
    .workspace {
      display: grid;
      grid-template-columns: minmax(300px, 1fr) minmax(250px, 330px);
      gap: 0 18px;
      align-items: start;
      padding: 0 18px 56px;
    }
    .remote-col,
    .secondary-col {
      display: block;
      min-width: 0;
    }
    .secondary-col {
      position: sticky;
      top: 18px;
    }
    .preview {
      margin-inline: 0;
    }
  }
  @media (min-width: 1360px) {
    .workspace {
      grid-template-columns: minmax(620px, 1fr) 400px;
      gap: 0 32px;
      padding-inline: 32px;
    }
  }
</style>
