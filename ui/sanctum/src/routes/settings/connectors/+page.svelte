<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/state';
  import { _ } from 'svelte-i18n';
  import { siObsstudio, siYoutube, siFacebook, siBlackmagicdesign, siDiscord } from 'simple-icons';
  import {
    PageHeader,
    SectionLabel,
    List,
    Glyph,
    Dot,
    Toggle,
    Field,
    Button,
    Segmented,
    Stat,
    OverviewCell,
    DiscoveryPanel,
  } from '@metocast/design-system';
  import {
    fetchConnectorStatuses,
    fetchConnectorConfig,
    saveConnectorConfig,
    connectObs,
    disconnectObs,
    fetchObsStreamSettings,
    applyObsStreamSettings,
    fetchYouTubeStreamKey,
    fetchFacebookStreamKey,
    fetchDevices,
    triggerDiscover,
    type ConnectorName,
    type ConnectorConfigMap,
    type BroadlinkDevice,
  } from '@metocast/core-client';
  import LoginSheet from '$lib/LoginSheet.svelte';
  import { pushToast } from '$lib/notifications.svelte';
  import NotifBell from '$lib/NotifBell.svelte';

  type ConnState = 'disconnected' | 'connecting' | 'connected' | 'error';
  type Category = 'broadcast' | 'streaming' | 'devices' | 'future';
  type FieldSpec = { key: string; type: 'text' | 'number' | 'password'; secret: boolean };

  interface ConnMeta {
    id: ConnectorName;
    name: string;
    cat: Category;
    supported: boolean;
    char?: string;
    brand?: string;
    fields: FieldSpec[];
  }

  const fld = (key: string, type: FieldSpec['type'] = 'text', secret = false): FieldSpec => ({ key, type, secret });

  const CONNECTORS: ConnMeta[] = [
    { id: 'obs', name: 'OBS Studio', cat: 'broadcast', supported: true, brand: siObsstudio.path, fields: [fld('host'), fld('port', 'number'), fld('password', 'password', true)] },
    { id: 'youtube', name: 'YouTube', cat: 'streaming', supported: true, brand: siYoutube.path, fields: [fld('clientId'), fld('clientSecret', 'password', true)] },
    { id: 'facebook', name: 'Facebook', cat: 'streaming', supported: true, brand: siFacebook.path, fields: [fld('appId'), fld('appSecret', 'password', true), fld('pageId')] },
    { id: 'broadlink', name: 'Broadlink RF/IR', cat: 'devices', supported: true, char: '⌁', fields: [] },
    { id: 'vmix', name: 'vMix', cat: 'future', supported: false, char: '▣', fields: [fld('host'), fld('port', 'number')] },
    { id: 'atem', name: 'Blackmagic ATEM', cat: 'future', supported: false, brand: siBlackmagicdesign.path, fields: [fld('host'), fld('port', 'number')] },
    { id: 'discord', name: 'Discord', cat: 'future', supported: false, brand: siDiscord.path, fields: [fld('webhookUrl', 'password', true)] },
  ];

  const CATS: Category[] = ['broadcast', 'streaming', 'devices', 'future'];

  const builders: { [K in ConnectorName]: (form: Record<string, string>, enabled: boolean) => ConnectorConfigMap[K] } = {
    obs: (form, enabled) => ({ enabled, host: form.host ?? '', port: Number(form.port) || 0, password: form.password || null }),
    vmix: (form, enabled) => ({ enabled, host: form.host ?? '', port: Number(form.port) || 0 }),
    atem: (form, enabled) => ({ enabled, host: form.host ?? '', port: Number(form.port) || 0 }),
    broadlink: (_form, enabled) => ({ enabled }),
    youtube: (form, enabled) => ({ enabled, clientId: form.clientId ?? '', clientSecret: form.clientSecret ?? '' }),
    facebook: (form, enabled) => ({ enabled, appId: form.appId ?? '', appSecret: form.appSecret ?? '', pageId: form.pageId ?? '' }),
    discord: (form, enabled) => ({ enabled, webhookUrl: form.webhookUrl ?? '' }),
    szentiras: (form, enabled) => ({ enabled, apiKey: form.apiKey ?? '' }),
  };

  let statuses = $state<Partial<Record<ConnectorName, ConnState>>>({});
  let enabled = $state<Record<string, boolean>>({});
  let forms = $state<Record<string, Record<string, string>>>(
    Object.fromEntries(CONNECTORS.map((c) => [c.id, Object.fromEntries(c.fields.map((f) => [f.key, '']))])),
  );
  let secretKept = $state<Record<string, string[]>>({});
  let expanded = $state<ConnectorName | null>('obs');
  let outcome = $state<Record<string, string>>({});
  let loginProvider = $state<'youtube' | 'facebook' | null>(null);

  let destination = $state<'youtube' | 'facebook'>('youtube');
  let rtmp = $state('');
  let devices = $state<BroadlinkDevice[]>([]);
  let scanning = $state(false);

  function fail(meta: ConnMeta, titleKey: string, message: string) {
    pushToast({ kind: $_('toast.connector'), source: meta.name, title: $_(titleKey), body: message, tone: 'error' });
  }

  const byCat = (cat: Category) => CONNECTORS.filter((c) => c.cat === cat);
  const enabledCount = $derived(CONNECTORS.filter((c) => enabled[c.id]).length);
  const readyCount = $derived(CONNECTORS.filter((c) => c.supported && enabled[c.id] && statuses[c.id] !== 'connected').length);
  const liveCount = $derived(CONNECTORS.filter((c) => statuses[c.id] === 'connected').length);
  const futureCount = $derived(CONNECTORS.filter((c) => !c.supported).length);
  const obsConnected = $derived(statuses.obs === 'connected');
  const ytLinked = $derived(statuses.youtube === 'connected');

  function dotColor(meta: ConnMeta): string {
    if (!enabled[meta.id] || !meta.supported) return 'var(--status-off)';
    switch (statuses[meta.id]) {
      case 'connected': return 'var(--status-ok)';
      case 'connecting': return 'var(--status-warn)';
      case 'error': return 'var(--status-error)';
      default: return 'var(--status-off)';
    }
  }

  function detailText(meta: ConnMeta): string {
    if (!enabled[meta.id]) return $_('conn.disabled');
    if (meta.id === 'obs') {
      const f = forms.obs ?? {};
      const state = statuses.obs ?? 'disconnected';
      return `${f.host || 'localhost'}:${f.port || '4455'} · ${state}`;
    }
    return $_(`conn.descriptor.${meta.id}`);
  }

  async function loadConfig(meta: ConnMeta) {
    const cfg = await fetchConnectorConfig(meta.id);
    const form: Record<string, string> = {};
    const kept: string[] = [];
    for (const [k, v] of Object.entries(cfg)) {
      if (k === 'enabled') enabled[meta.id] = v === true;
      else if (k.endsWith('Set')) { if (v === true) kept.push(k.slice(0, -3)); }
      else if (typeof v === 'string' || typeof v === 'number') form[k] = String(v);
    }
    forms[meta.id] = form;
    secretKept[meta.id] = kept;
  }

  async function refreshStatuses() {
    try {
      const s = await fetchConnectorStatuses();
      const next: Partial<Record<ConnectorName, ConnState>> = {};
      for (const c of CONNECTORS) next[c.id] = s[c.id]?.type;
      statuses = next;
    } catch {
      /* offline */
    }
  }

  let poll: ReturnType<typeof setInterval> | undefined;
  onMount(async () => {
    await refreshStatuses();
    await Promise.all(CONNECTORS.map((c) => loadConfig(c).catch(() => {})));
    poll = setInterval(refreshStatuses, 4000);
    const requested = page.url.searchParams.get('open') as ConnectorName | null;
    const meta = requested && CONNECTORS.find((c) => c.id === requested);
    if (meta) await open(meta);
  });
  onDestroy(() => clearInterval(poll));

  async function toggle(meta: ConnMeta) {
    if (!meta.supported) return;
    const next = !enabled[meta.id];
    enabled[meta.id] = next;
    try {
      await saveConnectorConfig(meta.id, builders[meta.id](forms[meta.id] ?? {}, next));
      await refreshStatuses();
    } catch (e) {
      enabled[meta.id] = !next;
      fail(meta, 'conn.toast.toggleFail', String(e));
    }
  }

  async function open(meta: ConnMeta) {
    expanded = expanded === meta.id ? null : meta.id;
    if (expanded !== meta.id) return;
    outcome[meta.id] = '';
    if (meta.id === 'broadlink') devices = await fetchDevices().catch(() => []);
  }

  async function save(meta: ConnMeta) {
    try {
      await saveConnectorConfig(meta.id, builders[meta.id](forms[meta.id] ?? {}, enabled[meta.id] ?? false));
      outcome[meta.id] = $_('conn.saved');
      await loadConfig(meta);
      await refreshStatuses();
    } catch (e) {
      outcome[meta.id] = $_('conn.saveFailed', { values: { error: String(e) } });
      fail(meta, 'conn.toast.saveFail', String(e));
    }
  }

  const obsMeta = CONNECTORS.find((c) => c.id === 'obs')!;
  async function toggleObs() {
    try {
      if (obsConnected) await disconnectObs();
      else await connectObs();
    } catch (e) {
      fail(obsMeta, obsConnected ? 'conn.toast.disconnectFail' : 'conn.toast.connectFail', String(e));
    }
    await refreshStatuses();
    if (statuses.obs !== 'connected') return;
    const s = await fetchObsStreamSettings().catch(() => null);
    if (s?.server) rtmp = s.server;
  }

  async function pickDestination(dest: 'youtube' | 'facebook') {
    destination = dest;
    if (!obsConnected) return;
    try {
      const key = dest === 'youtube' ? await fetchYouTubeStreamKey() : await fetchFacebookStreamKey();
      rtmp = key.rtmpUrl;
      const slash = rtmp.lastIndexOf('/');
      await applyObsStreamSettings(slash > 6 ? rtmp.slice(0, slash) : rtmp, slash > 6 ? rtmp.slice(slash + 1) : '');
    } catch (e) {
      fail(obsMeta, 'conn.toast.saveFail', String(e));
    }
  }

  const broadlinkMeta = CONNECTORS.find((c) => c.id === 'broadlink')!;
  async function scan() {
    scanning = true;
    try {
      await triggerDiscover();
      devices = await fetchDevices();
    } catch (e) {
      fail(broadlinkMeta, 'conn.toast.scanFail', String(e));
    } finally {
      scanning = false;
    }
  }
</script>

<PageHeader title={$_('screens.connectors.title')} back={{ label: $_('conn.back'), href: '/settings' }}>
  {#snippet trailing()}<NotifBell />{/snippet}
</PageHeader>

<div class="connectors-workspace">
  <div class="main-col">
    <section class="overview">
      <OverviewCell label={$_('conn.summary.live')} value={liveCount} color="var(--status-live)" />
      <OverviewCell label={$_('conn.summary.ready')} value={readyCount} color="var(--status-ok)" divider />
      <OverviewCell label={$_('conn.summary.future')} value={futureCount} color="var(--status-warn)" divider />
    </section>

    {#each CATS as cat (cat)}
      {@const items = byCat(cat)}
      {#if items.length}
        <SectionLabel hint={$_('conn.enabledHint', { values: { n: items.filter((c) => enabled[c.id]).length, total: items.length } })}>
          {$_(`conn.cat.${cat}`)}
        </SectionLabel>
        <List>
          {#each items as meta (meta.id)}
            {@const form = forms[meta.id] ?? {}}
            <div class="connector" class:muted={!enabled[meta.id]}>
              <button class="head" type="button" onclick={() => open(meta)}>
                {#if meta.brand}
                  <Glyph size={34}>
                    {#snippet mark()}<svg viewBox="0 0 24 24" width="19" height="19" fill="currentColor" aria-hidden="true"><path d={meta.brand} /></svg>{/snippet}
                  </Glyph>
                {:else}
                  <Glyph char={meta.char} size={34} />
                {/if}
                <span class="text">
                  <strong>{meta.name}</strong>
                  <em><Dot color={dotColor(meta)} size={5} />{detailText(meta)}</em>
                </span>
              </button>
              <Toggle checked={enabled[meta.id] ?? false} disabled={!meta.supported} label={meta.name} onchange={() => toggle(meta)} />
            </div>

            {#if expanded === meta.id}
              <section class="detail">
                {#if !meta.supported}<p>{$_('conn.lockedNote')}</p>{/if}

                {#each meta.fields as field (field.key)}
                  <Field
                    label={$_(`conn.field.${field.key}`)}
                    type={field.type}
                    bind:value={form[field.key]}
                    readonly={!meta.supported}
                    placeholder={field.secret && secretKept[meta.id]?.includes(field.key) ? '••••••••' : ''}
                    hint={field.secret && secretKept[meta.id]?.includes(field.key) ? $_('conn.secretKept') : ''}
                  />
                {/each}

                {#if meta.id === 'youtube' || meta.id === 'facebook'}
                  <div class="actions">
                    <Button variant="secondary" compact onclick={() => save(meta)}>{$_('conn.save')}</Button>
                    <Button variant="secondary" compact onclick={() => (loginProvider = meta.id === 'youtube' ? 'youtube' : 'facebook')}>
                      {statuses[meta.id] === 'connected' ? $_('conn.account') : $_('conn.login')}
                    </Button>
                  </div>
                {/if}

                {#if meta.id === 'obs'}
                  <div class="actions">
                    <Button variant="secondary" compact onclick={toggleObs}>
                      {obsConnected ? $_('conn.encoder.disconnect') : $_('conn.encoder.connect')}
                    </Button>
                    <Button variant="secondary" compact onclick={() => save(meta)}>{$_('conn.encoder.reconnect')}</Button>
                  </div>
                  <Segmented
                    compact
                    label={$_('conn.encoder.rtmp', { values: { dest: destination } })}
                    value={destination}
                    options={[{ value: 'youtube', label: 'YouTube' }, { value: 'facebook', label: 'Facebook' }]}
                    onchange={(v) => pickDestination(v as 'youtube' | 'facebook')}
                  />
                  <Field label={$_('conn.encoder.rtmp', { values: { dest: destination } })} value={rtmp} readonly />
                {/if}

                {#if meta.id === 'broadlink'}
                  <DiscoveryPanel
                    title={$_('conn.discovery.title')}
                    description={$_('conn.discovery.description')}
                    scanLabel={devices.length ? $_('conn.discovery.scanAgain') : $_('conn.discovery.scan')}
                    scanning={scanning}
                    scanningLabel={$_('conn.discovery.scanning')}
                    onscan={scan}
                  >
                    {#if devices.length === 0}
                      <p class="empty">{$_('conn.discovery.empty')}</p>
                    {:else}
                      <ul class="devices">
                        {#each devices as d (d.id)}
                          <li><strong>{d.name}</strong><code>{d.host}</code><em>{d.model ?? d.deviceType}</em></li>
                        {/each}
                      </ul>
                    {/if}
                  </DiscoveryPanel>
                {/if}

                {#if outcome[meta.id]}<p class="outcome">{outcome[meta.id]}</p>{/if}
              </section>
            {/if}
          {/each}
        </List>
      {/if}
    {/each}

    <p class="note">{$_('conn.note')}</p>
  </div>

  <aside class="ops-col">
    <section class="ops-card">
      <span class="kicker">{$_('conn.ops.kicker')}</span>
      <h2>{$_('conn.ops.route', { values: { dest: destination } })}</h2>
      <p class="route">
        <Dot color={obsConnected ? 'var(--status-ok)' : 'var(--status-off)'} size={6} />
        {obsConnected ? $_('conn.ops.obsReady') : $_('conn.ops.obsAttention')}
      </p>
      <div class="stats">
        <Stat label={$_('conn.ops.enabled')} value={enabledCount} unit={`/${CONNECTORS.length}`} />
        <Stat label={$_('conn.ops.ready')} value={readyCount} />
      </div>
      <dl>
        <div><dt>{$_('conn.ops.rtmp')}</dt><dd>{rtmp || '—'}</dd></div>
        <div><dt>{$_('conn.ops.deviceLayer')}</dt><dd>{devices.length ? $_('conn.ops.devicesFound', { values: { n: devices.length } }) : $_('conn.ops.awaitingScan')}</dd></div>
        <div><dt>{$_('conn.ops.authState')}</dt><dd>{ytLinked ? $_('conn.ops.ytLinked') : $_('conn.ops.ytNeedsLogin')}</dd></div>
      </dl>
    </section>
  </aside>
</div>
<div class="spacer"></div>

<LoginSheet bind:provider={loginProvider} />

<style>
  .overview {
    margin: 0 24px;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    border-block: 1px solid var(--border-hairline);
  }
  .connectors-workspace,
  .main-col {
    display: contents;
  }
  .ops-col {
    display: none;
  }
  .connector {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 16px 24px;
    border-bottom: 1px solid var(--border-hairline);
  }
  .muted {
    opacity: 0.72;
  }
  .head {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 14px;
    background: transparent;
    border: 0;
    padding: 0;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    font-family: inherit;
  }
  .text {
    min-width: 0;
  }
  strong {
    display: block;
    font-size: 15px;
    font-weight: 500;
    color: var(--text-primary);
  }
  em {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 4px;
    font-size: 12px;
    color: var(--text-secondary);
    font-style: normal;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .detail {
    padding: 14px 24px 18px 72px;
    border-bottom: 1px solid var(--border-hairline);
    display: grid;
    gap: 12px;
  }
  .detail p {
    margin: 0;
    font-family: var(--font-display);
    font-style: italic;
    color: var(--text-muted);
    font-size: 14px;
    line-height: 1.45;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .actions :global(button) {
    flex: 1;
  }
  .outcome {
    font-family: var(--font-body) !important;
    font-style: normal !important;
    font-size: 12px !important;
    color: var(--text-muted);
  }
  .empty {
    font-family: var(--font-display);
    font-style: italic;
    color: var(--text-muted);
    font-size: 14px;
  }
  .devices {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .devices li {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 2px 12px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border-hairline);
  }
  .devices code {
    font-family: var(--font-label);
    font-size: 11px;
    color: var(--text-muted);
  }
  .devices em {
    font-size: 11px;
    color: var(--text-muted);
  }
  .note {
    padding: 28px 32px 12px;
    text-align: center;
    margin: 0;
    font-family: var(--font-display);
    font-style: italic;
    color: var(--text-muted);
    font-size: 14px;
    line-height: 1.45;
  }
  .spacer {
    height: 120px;
  }
  .ops-card {
    background: var(--surface-raised);
    border-block: 1px solid var(--border-hairline);
    padding: 18px;
  }
  .kicker,
  dt {
    font-family: var(--font-label);
    font-size: 9px;
    letter-spacing: 1.5px;
    text-transform: uppercase;
    color: var(--text-muted);
  }
  h2 {
    margin: 8px 0 10px;
    font-family: var(--font-display);
    font-size: 28px;
    line-height: 1;
    font-weight: 500;
    color: var(--text-primary);
  }
  .route {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.4;
  }
  .stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 18px;
    margin: 18px 0;
    padding-block: 16px;
    border-block: 1px solid var(--border-hairline);
  }
  dl,
  dd {
    margin: 0;
  }
  dl div {
    padding: 12px 0;
    border-bottom: 1px solid var(--border-hairline);
  }
  dl div:last-child {
    border-bottom: 0;
  }
  dd {
    margin-top: 5px;
    color: var(--text-primary);
    font-size: 13px;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }
  @media (min-width: 760px) {
    .connectors-workspace {
      display: grid;
      grid-template-columns: minmax(286px, 1fr) minmax(206px, 260px);
      gap: 0 12px;
      align-items: start;
      max-width: 1040px;
      margin: 0 auto;
      padding: 0 14px 64px;
    }
    .main-col,
    .ops-col {
      display: block;
      min-width: 0;
    }
    .ops-col {
      position: sticky;
      top: 18px;
    }
    .overview {
      margin: 0;
    }
    .connector {
      padding-inline: 18px;
    }
    .detail {
      padding-left: 18px;
      padding-right: 18px;
    }
    .note {
      padding-bottom: 0;
    }
    .spacer {
      height: 0;
    }
  }
  @media (min-width: 1360px) {
    .connectors-workspace {
      grid-template-columns: minmax(620px, 1fr) 360px;
      gap: 0 30px;
      max-width: 1220px;
      padding-inline: 32px;
    }
    .detail {
      padding-left: 72px;
      padding-right: 24px;
    }
  }
</style>
