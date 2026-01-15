<script lang="ts">
	import { cn } from '$lib/utils.js';

	export let className: string = '';
</script>

<div class={cn('flex flex-col space-y-1.5 p-6', className)}>
	<slot />
</div>