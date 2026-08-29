<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { NotificationBell } from '@metocast/design-system';
  import { unreadCount, topTier, openCentre } from '$lib/notifications.svelte';
</script>

<NotificationBell
  label={$_('notif.bell', { values: { n: unreadCount() } })}
  unread={unreadCount()}
  tier={topTier()}
  onclick={openCentre}
/>
