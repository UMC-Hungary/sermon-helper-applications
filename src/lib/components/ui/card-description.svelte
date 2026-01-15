<script lang="ts">
	import { cn } from '$lib/utils.js';

	export let className: string = '';
</script>

<p class={cn('text-sm text-muted-foreground', className)}>
	<slot />
</p>