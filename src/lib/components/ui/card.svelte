<script lang="ts">
	export let className: string = '';

	const baseClasses = "rounded-lg border bg-card text-card-foreground shadow-sm";
	const computedClass = className ? `${baseClasses} ${className}` : baseClasses;
</script>

<div class={computedClass}>
	<slot />
</div>