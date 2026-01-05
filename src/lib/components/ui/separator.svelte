<script lang="ts">
	export let className: string = '';
	export let orientation: 'horizontal' | 'vertical' = 'horizontal';

	const baseClasses = "bg-border shrink-0 border data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-px";
	const computedClass = className ? `${baseClasses} ${className}` : baseClasses;
</script>

<div
	role="separator"
	aria-orientation={orientation}
	class={computedClass}
	{...$$restProps}
></div>