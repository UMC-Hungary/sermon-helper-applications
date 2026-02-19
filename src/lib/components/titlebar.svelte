<script lang="ts">
	import { Minus, Square, X } from 'lucide-svelte';
	import { browser } from '$app/environment';

	const isMac = browser && navigator.userAgent.includes('Mac');

	async function minimize() {
		const { getCurrentWindow } = await import('@tauri-apps/api/window');
		getCurrentWindow().minimize();
	}

	async function toggleMaximize() {
		const { getCurrentWindow } = await import('@tauri-apps/api/window');
		getCurrentWindow().toggleMaximize();
	}

	async function close() {
		const { getCurrentWindow } = await import('@tauri-apps/api/window');
		getCurrentWindow().close();
	}

	async function handleMouseDown(e: MouseEvent) {
		// Only handle left mouse button on the drag region itself (not child buttons)
		if (e.buttons === 1 && e.target === e.currentTarget) {
			if (e.detail === 2) {
				await toggleMaximize();
			} else {
				const { getCurrentWindow } = await import('@tauri-apps/api/window');
				getCurrentWindow().startDragging();
			}
		}
	}
</script>

<div
	data-tauri-drag-region
	class="fixed top-0 left-0 right-0 z-[9999] h-14 flex items-center select-none"
	class:justify-end={!isMac}
	class:pl-[70px]={isMac}
	role="toolbar"
	tabindex={-1}
	onmousedown={handleMouseDown}
>
	{#if !isMac}
		<div class="flex h-full" style="-webkit-app-region: no-drag">
			<button
				class="inline-flex items-center justify-center w-12 h-full hover:bg-muted-foreground/10 transition-colors"
				onclick={minimize}
				aria-label="Minimize"
			>
				<Minus class="h-4 w-4 text-foreground/70" />
			</button>
			<button
				class="inline-flex items-center justify-center w-12 h-full hover:bg-muted-foreground/10 transition-colors"
				onclick={toggleMaximize}
				aria-label="Maximize"
			>
				<Square class="h-3 w-3 text-foreground/70" />
			</button>
			<button
				class="inline-flex items-center justify-center w-12 h-full hover:bg-red-500/80 hover:text-white transition-colors"
				onclick={close}
				aria-label="Close"
			>
				<X class="h-4 w-4 text-foreground/70" />
			</button>
		</div>
	{/if}
</div>
