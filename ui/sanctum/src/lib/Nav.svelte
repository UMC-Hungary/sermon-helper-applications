<script lang="ts">
  import { page } from '$app/stores';
  import { _ } from 'svelte-i18n';
  import { Icon, Lockup } from '@metocast/design-system';

  // One nav over the primary destinations. Event editor lives under events,
  // connectors under settings, so those keep their parent tab active.
  const tabs = [
    { href: '/', icon: 'home', key: 'dashboard', active: (p: string) => p === '/' },
    { href: '/events', icon: 'calendar', key: 'events', active: (p: string) => p.startsWith('/events') },
    { href: '/presentations', icon: 'slides', key: 'slides', active: (p: string) => p.startsWith('/presentations') },
    { href: '/settings', icon: 'gear', key: 'settings', active: (p: string) => p.startsWith('/settings') },
  ] as const;

  const path = $derived($page.url.pathname);
</script>

<nav>
  <div class="brand"><Lockup name="Metocast" /></div>
  {#each tabs as tab (tab.href)}
    <a
      href={tab.href}
      class:active={tab.active(path)}
      aria-current={tab.active(path) ? 'page' : undefined}
    >
      <Icon name={tab.icon} size={22} />
      <span>{$_(`nav.${tab.key}`)}</span>
    </a>
  {/each}
</nav>

<style>
  nav {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: var(--z-nav);
    display: flex;
    justify-content: space-around;
    align-items: center;
    background: var(--surface-base);
    border-top: 1px solid color-mix(in srgb, var(--text-primary) 12%, transparent);
    padding: 10px 6px max(14px, env(safe-area-inset-bottom, 14px));
  }

  .brand {
    display: none;
  }

  a {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 4px;
    text-decoration: none;
    color: var(--text-muted);
  }

  a.active {
    color: var(--text-primary);
  }

  span {
    font-family: var(--font-label);
    font-size: 9px;
    letter-spacing: 1.2px;
    text-transform: uppercase;
  }

  a.active span {
    font-weight: 500;
  }

  /* Side navigation above the mobile breakpoint. */
  @media (min-width: 980px) {
    nav {
      top: 0;
      right: auto;
      bottom: 0;
      width: 226px;
      flex-direction: column;
      align-items: stretch;
      justify-content: flex-start;
      gap: 4px;
      padding: 22px 14px;
      border-top: 0;
      border-right: 1px solid color-mix(in srgb, var(--text-primary) 12%, transparent);
    }

    .brand {
      display: block;
      padding: 2px 8px 22px;
      margin-bottom: 12px;
      border-bottom: 1px solid color-mix(in srgb, var(--text-primary) 12%, transparent);
    }

    a {
      flex: 0 0 auto;
      min-height: 44px;
      flex-direction: row;
      justify-content: flex-start;
      gap: 12px;
      padding: 10px 9px;
      border-left: 2px solid transparent;
      /* Square, per the reference — a flat background with a left accent, no rounding. */
    }

    a.active {
      background: var(--surface-raised);
      border-left-color: var(--text-primary);
    }

    span {
      font-size: 10px;
      letter-spacing: 1.4px;
    }
  }
</style>
