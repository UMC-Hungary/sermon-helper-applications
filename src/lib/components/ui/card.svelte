<script lang="ts">
	import { cn } from "$lib/utils";
	import type { HTMLAttributes } from "svelte/elements";

	type CardProps = HTMLAttributes<HTMLDivElement> & {
		clickable?: boolean;
	};

	export let className: string = "";
	export let clickable: boolean = false;

	const baseClasses = "bg-card text-card-foreground flex flex-col gap-6 rounded-xl border py-6 shadow-sm";
	const clickableClasses = clickable ? "cursor-pointer hover:bg-accent/50 transition-colors" : "";
</script>

<div
	data-slot="card"
	class={cn(baseClasses, clickableClasses, className)}
	{...$$restProps}
>
	<!-- Header Slot: Contains title, description, and optional action -->
	{#if $$slots.header || $$slots.title || $$slots.description || $$slots.action}
		<div
			data-slot="card-header"
			class="@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-2 px-6 has-data-[slot=card-action]:grid-cols-[1fr_auto] [.border-b]:pb-6"
		>
			<!-- Title Slot -->
			{#if $$slots.title}
				<div data-slot="card-title" class="leading-none font-semibold flex items-center gap-2">
					<slot name="title" />
				</div>
			{/if}

			<!-- Description Slot -->
			{#if $$slots.description}
				<div data-slot="card-description" class="text-muted-foreground text-sm">
					<slot name="description" />
				</div>
			{/if}

			<!-- Action Slot (positioned to the right) -->
			{#if $$slots.action}
				<div
					data-slot="card-action"
					class="col-start-2 row-span-2 row-start-1 self-start justify-self-end"
				>
					<slot name="action" />
				</div>
			{/if}

			<!-- Default header content -->
			<slot name="header" />
		</div>
	{/if}

	<!-- Content Slot -->
	{#if $$slots.content}
		<div data-slot="card-content" class="px-6">
			<slot name="content" />
		</div>
	{/if}

	<!-- Footer Slot -->
	{#if $$slots.footer}
		<div data-slot="card-footer" class="flex items-center px-6 [.border-t]:pt-6">
			<slot name="footer" />
		</div>
	{/if}

	<!-- Default slot for backward compatibility -->
	<slot />
</div>
