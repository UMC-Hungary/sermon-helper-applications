<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { _, locale } from 'svelte-i18n';
  import {
    PageHeader,
    FormSection,
    TextArea,
    NativeDateInput,
    ReferenceInput,
    Segmented,
    ToggleRow,
    StickyActionBar,
    Skeleton,
  } from '@metocast/design-system';
  import type { ReferenceResult } from '@metocast/design-system';
  import { getEvent, createEvent, updateEvent, bibleApi } from '@metocast/core-client';
  import type { Event, BibleVerse } from '@metocast/core-client/schemas/event';
  import { goto } from '$app/navigation';
  import { notify } from '$lib/notifications.svelte';
  import { dateLong, toDateInput, toTimeInput, fromDateTimeInput } from '$lib/format';

  let { eventId }: { eventId?: string } = $props();

  const TITLE_LIMIT = 100;
  const DESC_LIMIT = 5000;
  const TRANSLATION = 'RUF_v2';

  interface RefData {
    ref: string;
    invalid?: boolean;
    translation?: string;
    verses: BibleVerse[];
  }

  let loading = $state(false);
  let title = $state('');
  let date = $state('');
  let time = $state('');
  let speaker = $state('');
  let description = $state('');
  let privacy = $state<'public' | 'unlisted' | 'private'>('public');
  let autoUpload = $state(true);
  let textus = $state('');
  let leckio = $state('');
  let textusData = $state<RefData | null>(null);
  let leckioData = $state<RefData | null>(null);
  let existingConnections = $state<Event['connections']>([]);

  const loc = $derived($locale ?? 'en');

  // Hard cap the title as the reference does, without dropping what the user typed elsewhere.
  $effect(() => {
    if (title.length > TITLE_LIMIT) title = title.slice(0, TITLE_LIMIT);
  });

  const titleRemaining = $derived(TITLE_LIMIT - title.length);
  const autoTitle = $derived(
    [date ? dateLong(fromDateTimeInput(date, time), loc) : '', textus.trim(), title.trim(), speaker.trim() ? `— ${speaker.trim()}` : '']
      .filter(Boolean)
      .join(' · ')
      .replace('· —', '—'),
  );
  const fields = $derived([
    { key: 'date', on: !!date },
    { key: 'textus', on: !!textus.trim() },
    { key: 'title', on: !!title.trim() },
    { key: 'speaker', on: !!speaker.trim() },
  ]);

  function toResult(data: RefData | null): ReferenceResult | null {
    if (!data) return null;
    if (data.invalid) return { invalid: true };
    return {
      verses: data.verses.map((v) => ({ n: `${v.chapter}:${v.verse}`, text: v.text })),
      translation: data.translation,
    };
  }
  const textusResult = $derived(toResult(textusData));
  const leckioResult = $derived(toResult(leckioData));

  function statusMessage(data: RefData | null): string {
    if (!data || data.invalid || data.verses.length === 0) return '';
    return $_('editor.verseCount', { values: { n: data.verses.length, t: data.translation } });
  }

  // A debounced lookup that ignores its own stale responses, so a slower earlier
  // request can never overwrite a newer one (task 7.6).
  function makeLookup(set: (d: RefData | null) => void) {
    let seq = 0;
    let timer: ReturnType<typeof setTimeout>;
    return (ref: string) => {
      clearTimeout(timer);
      const q = ref.trim();
      if (!q) {
        set(null);
        return;
      }
      const my = ++seq;
      timer = setTimeout(async () => {
        try {
          const { verses } = await bibleApi.fetchVerses(q, TRANSLATION);
          if (my !== seq) return;
          set(verses.length ? { ref: q, translation: TRANSLATION, verses } : { ref: q, invalid: true, verses: [] });
        } catch {
          if (my !== seq) return;
          set({ ref: q, invalid: true, verses: [] });
        }
      }, 500);
    };
  }
  const lookupTextus = makeLookup((d) => (textusData = d));
  const lookupLeckio = makeLookup((d) => (leckioData = d));

  $effect(() => lookupTextus(textus));
  $effect(() => lookupLeckio(leckio));

  const privacyOptions = $derived([
    { value: 'public' as const, label: $_('editor.privacy.public'), glyph: '◉', hint: $_('editor.privacy.publicHint') },
    { value: 'unlisted' as const, label: $_('editor.privacy.unlisted'), glyph: '◐', hint: $_('editor.privacy.unlistedHint') },
    { value: 'private' as const, label: $_('editor.privacy.private'), glyph: '○', hint: $_('editor.privacy.privateHint') },
  ]);

  function nextSundayDate(): string {
    const now = new Date();
    const daysUntilSunday = (7 - now.getDay()) % 7;
    const sunday = new Date(now.getTime() + daysUntilSunday * 24 * 60 * 60 * 1000);
    return toDateInput(sunday.toISOString());
  }

  onMount(async () => {
    if (!eventId) {
      title = get(_)('editor.defaultTitle');
      date = nextSundayDate();
      time = '10:00';
      return;
    }
    loading = true;
    try {
      const e = await getEvent(eventId);
      title = e.title;
      date = toDateInput(e.dateTime);
      time = toTimeInput(e.dateTime);
      speaker = e.speaker;
      description = e.description;
      autoUpload = e.autoUploadEnabled;
      existingConnections = e.connections;
      const yt = e.connections.find((c) => c.platform === 'youtube');
      if (yt?.privacyStatus === 'unlisted' || yt?.privacyStatus === 'private' || yt?.privacyStatus === 'public') privacy = yt.privacyStatus;
      const t = e.bibleReferences.find((r) => r.type === 'textus');
      const l = e.bibleReferences.find((r) => r.type === 'leckio');
      if (t) {
        textus = t.reference;
        textusData = { ref: t.reference, translation: t.translation, verses: t.verses };
      }
      if (l) {
        leckio = l.reference;
        leckioData = { ref: l.reference, translation: l.translation, verses: l.verses };
      }
    } catch {
      notify({ tier: 'error', kind: 'Event', source: 'Core', title: $_('editor.loadFailed') });
    } finally {
      loading = false;
    }
  });

  function buildRefs() {
    const out: { type: string; reference: string; translation?: string; verses?: { chapter: number; verse: number; text: string }[] }[] = [];
    if (textusData && !textusData.invalid && textus.trim())
      out.push({ type: 'textus', reference: textus.trim(), translation: textusData.translation, verses: textusData.verses.map((v) => ({ chapter: v.chapter, verse: v.verse, text: v.text })) });
    if (leckioData && !leckioData.invalid && leckio.trim())
      out.push({ type: 'leckio', reference: leckio.trim(), translation: leckioData.translation, verses: leckioData.verses.map((v) => ({ chapter: v.chapter, verse: v.verse, text: v.text })) });
    return out;
  }

  function buildConnections() {
    const others = existingConnections.filter((c) => c.platform !== 'youtube');
    const conns = others.map((c) => ({ platform: c.platform, privacy_status: c.privacyStatus ?? undefined }));
    conns.push({ platform: 'youtube', privacy_status: privacy });
    return conns;
  }

  let saving = $state(false);
  const canSave = $derived(title.trim().length > 0 && !!date && !saving);

  async function save() {
    if (!canSave) return;
    saving = true;
    const payload = {
      title: title.trim(),
      date_time: fromDateTimeInput(date, time),
      speaker: speaker.trim(),
      description: description.trim(),
      auto_upload_enabled: autoUpload,
      connections: buildConnections(),
      bible_references: buildRefs(),
    };
    try {
      if (eventId) await updateEvent(eventId, payload);
      else await createEvent(payload);
      goto('/events');
    } catch (e) {
      // Report and keep every entered value (task 7.2).
      notify({ tier: 'error', kind: 'Event', source: 'Core', title: $_('editor.saveFailed'), body: e instanceof Error ? e.message : undefined });
      saving = false;
    }
  }
</script>

<PageHeader
  back={{ label: $_('editor.back'), href: '/events' }}
  title={eventId ? $_('editor.editTitle') : $_('editor.newTitle')}
/>

{#if loading}
  <div class="pad"><Skeleton height="120px" lines={4} /></div>
{:else}
  <section class="preview sticky-preview">
    <small>{$_('editor.previewLabel')} <span class:warn={titleRemaining < 10}>{title.length}/{TITLE_LIMIT}</span></small>
    <p>{autoTitle || $_('editor.previewEmpty')}</p>
    <div class="tags">
      {#each fields as f (f.key)}<em class:on={f.on}>{$_(`editor.tag.${f.key}`)}</em>{/each}
    </div>
  </section>

  <form onsubmit={(e) => { e.preventDefault(); save(); }}>
    <div class="main-col">
      <FormSection number="01" label={$_('editor.details')}>
        <div class="field">
          <TextArea bind:value={title} rows={3} label={$_('editor.eventTitle')} placeholder={$_('editor.titlePlaceholder')} invalid={titleRemaining < 0} />
          <small class:warn={titleRemaining < 10}>{title.length} / {TITLE_LIMIT}</small>
        </div>
        <div class="two">
          <div class="field">
            <span class="cap">{$_('editor.date')}</span>
            <NativeDateInput type="date" bind:value={date} label={date ? dateLong(fromDateTimeInput(date, time), loc) : $_('editor.pickDate')} icon="calendar" accessibleName={$_('editor.date')} />
          </div>
          <div class="field">
            <span class="cap">{$_('editor.time')}</span>
            <NativeDateInput type="time" bind:value={time} label={time || $_('editor.pickTime')} icon="clock" accessibleName={$_('editor.time')} />
          </div>
        </div>
        <div class="field">
          <span class="cap">{$_('editor.speaker')}</span>
          <TextArea bind:value={speaker} rows={1} label={$_('editor.speaker')} placeholder={$_('editor.speakerPlaceholder')} />
        </div>
      </FormSection>

      <FormSection number="02" label={$_('editor.scripture')} hint={$_('editor.scriptureHint')}>
        <ReferenceInput
          bind:value={textus}
          label={$_('editor.textus')}
          rank={$_('editor.primary')}
          placeholder={$_('editor.textusPlaceholder')}
          result={textusResult}
          statusMessage={statusMessage(textusData)}
          errorMessage={$_('editor.refNotFound')}
          notFoundLabel={$_('editor.refNotFound')}
        />
        <ReferenceInput
          bind:value={leckio}
          label={$_('editor.leckio')}
          rank={$_('editor.secondary')}
          placeholder={$_('editor.leckioPlaceholder')}
          result={leckioResult}
          statusMessage={statusMessage(leckioData)}
          errorMessage={$_('editor.refNotFound')}
          notFoundLabel={$_('editor.refNotFound')}
        />
      </FormSection>

      <FormSection number="03" label={$_('editor.description')} hint={$_('editor.descriptionHint')}>
        <div class="field">
          <TextArea bind:value={description} rows={5} label={$_('editor.description')} placeholder={$_('editor.descriptionPlaceholder')} />
          <small class:warn={description.length > DESC_LIMIT}>{description.length} / {DESC_LIMIT}</small>
        </div>
      </FormSection>
    </div>

    <aside class="publish-col">
      <FormSection number="04" label={$_('editor.privacyLabel')}>
        <Segmented bind:value={privacy} options={privacyOptions} label={$_('editor.privacyLabel')} />
      </FormSection>

      <FormSection number="05" label={$_('editor.recording')} last>
        <ToggleRow label={$_('editor.autoUpload')} sub={$_('editor.autoUploadHint')} bind:checked={autoUpload} />
      </FormSection>
    </aside>
  </form>

  <p class="note">{$_('editor.footNote')}</p>
  <StickyActionBar
    primary={eventId ? $_('editor.saveEdit') : $_('editor.saveNew')}
    secondary={$_('editor.cancel')}
    primaryDisabled={!canSave}
    onprimary={save}
    onsecondary={() => goto('/events')}
    bottom="0"
  />
{/if}

<style>
  .pad {
    padding: 24px;
  }
  form {
    padding: 0 20px;
  }
  .two {
    display: grid;
    grid-template-columns: 1.35fr 1fr;
    gap: 8px;
  }
  .field {
    margin-bottom: 12px;
  }
  .cap {
    display: block;
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 1.2px;
    text-transform: uppercase;
    color: var(--text-muted);
    margin-bottom: 6px;
  }
  small {
    display: block;
    font-family: var(--font-mono);
    font-size: 9px;
    color: var(--text-muted);
    letter-spacing: 1.2px;
    text-align: right;
    margin-top: 6px;
  }
  small.warn,
  .preview span.warn {
    color: var(--status-warn);
  }
  .preview {
    padding-block: 14px;
    padding-inline: 16px;
    background: var(--text-primary);
    color: var(--surface-base, var(--surface-outside));
  }
  .sticky-preview {
    position: sticky;
    top: 0;
    z-index: 5;
    margin: 0 20px 20px;
  }
  .preview small {
    color: color-mix(in srgb, currentColor 55%, transparent);
    text-transform: uppercase;
    text-align: left;
    margin: 0 0 8px;
    display: flex;
    justify-content: space-between;
  }
  .preview p {
    margin: 0;
    font-family: var(--font-display);
    font-size: 18px;
    line-height: 1.3;
    font-weight: 500;
    overflow-wrap: anywhere;
  }
  .tags {
    display: flex;
    gap: 14px;
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid color-mix(in srgb, currentColor 14%, transparent);
  }
  .preview em {
    font-family: var(--font-mono);
    font-size: 9px;
    color: color-mix(in srgb, currentColor 40%, transparent);
    letter-spacing: 1.3px;
    text-transform: uppercase;
    font-style: normal;
  }
  .preview em.on {
    color: currentColor;
  }
  .note {
    padding: 24px 32px;
    text-align: center;
    color: var(--text-muted);
    font-family: var(--font-display);
    font-style: italic;
    font-size: 13px;
    line-height: 1.5;
  }
.main-col,
  .publish-col {
    display: contents;
  }
  @media (min-width: 760px) {
    form {
      display: grid;
      grid-template-columns: minmax(340px, 1fr) minmax(280px, 380px);
      gap: 0 22px;
      padding-inline: 24px;
      max-width: 1120px;
      margin: 0 auto;
      align-items: start;
    }
    .main-col,
    .publish-col {
      display: block;
      min-width: 0;
    }
    .publish-col {
      position: sticky;
      top: 18px;
    }
    .note {
      max-width: 620px;
      margin: 0 auto;
    }
    .sticky-preview {
      max-width: 1120px;
      margin: 0 auto 20px;
      padding-inline: 24px;
    }
  }
  @media (min-width: 1360px) {
    form {
      grid-template-columns: minmax(560px, 1fr) 400px;
      gap: 0 32px;
      padding-inline: 32px;
    }
    .sticky-preview {
      max-width: 1260px;
      padding-inline: 32px;
    }
  }
</style>
