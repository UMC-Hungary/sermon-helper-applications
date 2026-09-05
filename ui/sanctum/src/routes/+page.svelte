<script lang="ts">
  import { onMount } from 'svelte';
  import { _, locale } from 'svelte-i18n';
  import {
    PageHeader,
    Lockup,
    Dot,
    Stat,
    SectionLabel,
    List,
    Row,
    DateBlock,
    TextIcon,
    Skeleton,
    ErrorState,
    EmptyState,
  } from '@metocast/design-system';
  import { listEvents, fetchConnectorStatuses } from '@metocast/core-client';
  import type { EventSummary } from '@metocast/core-client/schemas/event';
  import { live } from '$lib/live.svelte';
  import { monthAbbr, dayNum, timeShort, eventTitle } from '$lib/format';
  import NotifBell from '$lib/NotifBell.svelte';

  let events = $state<EventSummary[]>([]);
  let connected = $state(0);
  let phase = $state<'loading' | 'ready' | 'error'>('loading');

  const loc = $derived($locale ?? 'en');
  const now = new Date();
  const hour = now.getHours() + now.getMinutes() / 60;
  const greeting =
    hour >= 23 || hour < 4.5
      ? 'greetingNight'
      : hour < 8
        ? 'greetingEarly'
        : hour < 12
          ? 'greetingMorning'
          : hour < 18
            ? 'greetingAfternoon'
            : 'greetingEvening';
  const nextEvent = $derived(
    events
      .filter((e) => new Date(e.dateTime).getTime() >= Date.now())
      .sort((a, b) => a.dateTime.localeCompare(b.dateTime))[0],
  );

  async function load() {
    phase = 'loading';
    try {
      events = await listEvents();
      phase = 'ready';
    } catch {
      phase = 'error';
    }
    try {
      const s = await fetchConnectorStatuses();
      connected = Object.values(s).filter((c) => c?.type === 'connected').length;
    } catch {
      /* offline — leave count at 0 */
    }
  }

  onMount(load);
</script>

<PageHeader title={$_(`dash.${greeting}`)} eyebrowContent={eyebrow}>
  {#snippet trailing()}<NotifBell />{/snippet}
</PageHeader>
{#snippet eyebrow()}<Lockup name="Sanctum" markSize={16} fontSize={15} tracking={1.6} />{/snippet}

<div class="dashboard-grid">
  <div class="main-col">
    <section class="now">
      <article>
        <header>
          <p>
            <Dot
              color={live.streaming ? 'var(--status-live)' : 'var(--text-muted)'}
              size={8}
              pulse={live.streaming}
            />
            <span class:live={live.streaming}
              >{live.streaming ? $_('dash.onAir') : $_('dash.offAir')}</span
            >
          </p>
          <time>{live.streaming ? '00:42:18' : '—'}</time>
        </header>
        <h2>{live.streaming ? $_('dash.activeTitle') : $_('dash.noBroadcast')}</h2>
        <p class="sub">{live.streaming ? $_('dash.activeContext') : $_('dash.idleHint')}</p>
        <div class="stats">
          <Stat label={$_('dash.viewers')} value={live.streaming ? '1,284' : '—'} />
          <Stat
            label={$_('dash.bitrate')}
            value={live.streaming ? '6.2' : '—'}
            unit={live.streaming ? 'Mb/s' : ''}
          />
          <Stat
            label={$_('dash.dropped')}
            value={live.streaming ? '0.00' : '—'}
            unit={live.streaming ? '%' : ''}
          />
        </div>
      </article>
    </section>

    <SectionLabel hint={nextEvent ? timeShort(nextEvent.dateTime, loc) : ''}
      >{$_('dash.upNext')}</SectionLabel
    >
    {#if phase === 'loading'}
      <div class="pad"><Skeleton height="64px" /></div>
    {:else if phase === 'error'}
      <ErrorState
        title={$_('common.errorTitle')}
        body={$_('common.errorBody')}
        retryLabel={$_('common.retry')}
        onretry={load}
      />
    {:else if nextEvent}
      <List>
        <Row
          title={eventTitle(nextEvent)}
          meta={`${timeShort(nextEvent.dateTime, loc)} · ${nextEvent.speaker || $_('dash.noSpeaker')}`}
          href={`/events/${nextEvent.id}`}
          last
        >
          {#snippet icon()}<DateBlock
              month={monthAbbr(nextEvent.dateTime, loc)}
              day={dayNum(nextEvent.dateTime, loc)}
            />{/snippet}
        </Row>
      </List>
    {:else}
      <EmptyState title={$_('dash.noneScheduled')} hint={$_('dash.noneScheduledHint')} />
    {/if}
  </div>

  <aside class="side-col">
    <SectionLabel>{$_('dash.quickActions')}</SectionLabel>
    <List>
      <Row title={$_('dash.newEvent')} meta={$_('dash.newEventMeta')} href="/events/new">
        {#snippet icon()}<TextIcon char="+" />{/snippet}
      </Row>
      <Row
        title={$_('dash.presentations')}
        meta={$_('dash.presentationsMeta')}
        href="/presentations"
      >
        {#snippet icon()}<TextIcon char="▥" />{/snippet}
      </Row>
      <Row
        title={$_('dash.connectors')}
        meta={$_('dash.connectorsMeta', { values: { n: connected } })}
        href="/settings/connectors"
        last
      >
        {#snippet icon()}<TextIcon char="↳" />{/snippet}
      </Row>
    </List>
  </aside>
</div>

<style>
  .now {
    padding: 0 24px;
  }
  .pad {
    padding: 0 24px;
  }
  article {
    border: 1px solid color-mix(in srgb, var(--text-primary) 16%, transparent);
    padding: 20px 20px 22px;
  }
  header,
  header p {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  header p {
    gap: 9px;
    margin: 0;
  }
  header span,
  time {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 2px;
    text-transform: uppercase;
    color: var(--text-muted);
    font-weight: 500;
  }
  header span.live {
    color: var(--status-live);
  }
  time {
    font-size: 12px;
    letter-spacing: 0.5px;
    text-transform: none;
    color: var(--text-secondary);
  }
  h2 {
    margin: 16px 0 0;
    font-family: var(--font-display);
    font-size: 26px;
    line-height: 1.12;
    color: var(--text-primary);
    font-weight: 500;
  }
  .sub {
    margin: 4px 0 0;
    font-size: 13px;
    color: var(--text-secondary);
  }
  .stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    margin-top: 22px;
    padding-top: 18px;
    border-top: 1px solid color-mix(in srgb, var(--text-primary) 12%, transparent);
  }
  .dashboard-grid,
  .main-col,
  .side-col {
    display: contents;
  }

  @media (min-width: 760px) {
    .dashboard-grid {
      display: grid;
      grid-template-columns: minmax(320px, 1fr) minmax(260px, 340px);
      gap: 18px;
      padding: 0 18px 56px;
      align-items: start;
    }
    .main-col,
    .side-col {
      display: block;
      min-width: 0;
    }
    .now,
    .pad {
      padding-inline: 0;
    }
    .side-col {
      position: sticky;
      top: 18px;
    }
  }
  @media (min-width: 1360px) {
    .dashboard-grid {
      grid-template-columns: minmax(560px, 1fr) 360px;
      gap: 28px;
      padding-inline: 32px;
    }
  }
</style>
