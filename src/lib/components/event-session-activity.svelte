<script lang="ts">
	import { _ } from 'svelte-i18n';
	import type { SessionActivity } from '$lib/types/event';

	export let activities: SessionActivity[];

	function formatTime(timestamp: number): string {
		return new Date(timestamp).toLocaleTimeString(undefined, {
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit'
		});
	}
</script>

<div class="space-y-2">
	<h3 class="text-sm font-medium text-muted-foreground">{$_('eventSession.activity.title')}</h3>
	<div class="text-xs space-y-1 font-mono">
		{#each activities as activity}
			<div class="flex gap-2 {activity.type === 'SESSION_ERROR' ? 'text-destructive' : 'text-muted-foreground'}">
				<span class="shrink-0">{formatTime(activity.timestamp)}</span>
				<span>{$_(`eventSession.activity.${activity.type}`)}</span>
				{#if activity.message}
					<span class="truncate opacity-70">— {activity.message}</span>
				{/if}
			</div>
		{/each}
	</div>
</div>
