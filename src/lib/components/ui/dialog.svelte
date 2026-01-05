<script lang="ts">
	import { cn } from "$lib/utils";
	import type { HTMLAttributes } from "svelte/elements";

	type DialogRootProps = HTMLAttributes<HTMLDivElement>;
	type DialogTriggerProps = HTMLAttributes<HTMLButtonElement>;
	type DialogContentProps = HTMLAttributes<HTMLDivElement> & {
		forceMount?: boolean;
	};
	type DialogHeaderProps = HTMLAttributes<HTMLDivElement>;
	type DialogTitleProps = HTMLAttributes<HTMLHeadingElement>;
	type DialogDescriptionProps = HTMLAttributes<HTMLDivElement>;

	interface Dialog {
		Root: typeof DialogRoot;
		Trigger: typeof DialogTrigger;
		Content: typeof DialogContent;
		Header: typeof DialogHeader;
		Title: typeof DialogTitle;
		Description: typeof DialogDescription;
	}

	// Mock Dialog components for simplicity
	const DialogRoot = "div";
	const DialogTrigger = "button";
	const DialogContent = "div";
	const DialogHeader = "div";
	const DialogTitle = "h3";
	const DialogDescription = "div";

	export let open = false;
	export let onOpenChange = () => {};
	export let className: string = "";

	const baseClasses = "relative w-full";
</script>

<Dialog:Root {open} onOpenChange={onOpenChange} class={cn(baseClasses, className)}>
	<slot name="content" />
</Dialog:Root>