<script lang="ts">
	import { onMount } from 'svelte';
	import { _ } from 'svelte-i18n';
	import { open } from '@tauri-apps/plugin-dialog';
	import { pptFilter, pptResults, pptFolders, keynoteStatus } from '$lib/stores/presentations.js';
	import { sendWsCommand } from '$lib/ws/client.js';
	import {
		listFolders,
		addFolder,
		removeFolder,
		searchFiles,
		openFile,
		keynoteNext,
		keynotePrev,
		keynoteFirst,
		keynoteLast,
		keynoteStart,
		keynoteStop,
		keynoteCloseAll,
	} from '$lib/api/presentations.js';

	let loading = $state(false);
	let addingFolder = $state(false);

	onMount(async () => {
		const folders = await listFolders();
		pptFolders.set(folders);
	});

	async function handleAddFolder() {
		addingFolder = true;
		try {
			const selected = await open({ directory: true, multiple: false });
			if (!selected || typeof selected !== 'string') return;
			const name = selected.split('/').pop() ?? selected;
			const folder = await addFolder(selected, name);
			if (folder) {
				pptFolders.update((list) => [...list, folder]);
			}
		} finally {
			addingFolder = false;
		}
	}

	async function handleRemoveFolder(id: string) {
		await removeFolder(id);
		pptFolders.update((list) => list.filter((f) => f.id !== id));
	}

	function appendDigit(digit: string) {
		pptFilter.update((f) => f + digit);
		triggerSearch();
	}

	function handleBackspace() {
		pptFilter.update((f) => f.slice(0, -1));
		triggerSearch();
	}

	function handleClear() {
		pptFilter.set('');
		triggerSearch();
	}

	async function triggerSearch() {
		const filter = $pptFilter;
		if (!sendWsCommand('ppt.search', { filter })) {
			const files = await searchFiles(filter);
			pptResults.set(files);
		}
	}

	async function handleOpenSlot(path: string) {
		loading = true;
		try {
			if (!sendWsCommand('keynote.open', { file_path: path })) {
				await openFile(path);
			}
		} finally {
			loading = false;
		}
	}

	async function handleNext() {
		if (!sendWsCommand('keynote.next')) await keynoteNext();
	}
	async function handlePrev() {
		if (!sendWsCommand('keynote.prev')) await keynotePrev();
	}
	async function handleFirst() {
		if (!sendWsCommand('keynote.first')) await keynoteFirst();
	}
	async function handleLast() {
		if (!sendWsCommand('keynote.last')) await keynoteLast();
	}
	async function handleStart() {
		if (!sendWsCommand('keynote.start')) await keynoteStart();
	}
	async function handleStop() {
		if (!sendWsCommand('keynote.stop')) await keynoteStop();
	}
	async function handleCloseAll() {
		if (!sendWsCommand('keynote.close_all')) await keynoteCloseAll();
	}
</script>

<div class="presentations-page">
	<h1>{$_('presentations.title')}</h1>

	<!-- Folder Manager -->
	<section class="section">
		<h2>{$_('presentations.folders.title')}</h2>
		<ul class="folder-list">
			{#each $pptFolders as folder (folder.id)}
				<li class="folder-item">
					<span class="folder-name" title={folder.path}>{folder.name}</span>
					<span class="folder-path">{folder.path}</span>
					<button
						onclick={() => handleRemoveFolder(folder.id)}
						aria-label={$_('presentations.folders.remove')}
					>
						✕
					</button>
				</li>
			{/each}
		</ul>
		<button class="btn-primary" onclick={handleAddFolder} disabled={addingFolder}>
			{addingFolder ? $_('common.loading') : $_('presentations.folders.add')}
		</button>
	</section>

	<!-- File Search -->
	<section class="section">
		<h2>{$_('presentations.search.title')}</h2>
		<div class="filter-display">
			<span class="filter-value">{$pptFilter || '—'}</span>
		</div>
		<div class="digit-pad">
			{#each ['1','2','3','4','5','6','7','8','9','0'] as digit}
				<button class="digit-btn" onclick={() => appendDigit(digit)}>{digit}</button>
			{/each}
			<button class="digit-btn backspace-btn" onclick={handleBackspace}>⌫</button>
			<button class="digit-btn clear-btn" onclick={handleClear}>CLR</button>
		</div>
	</section>

	<!-- Result Slots -->
	<section class="section">
		<h2>{$_('presentations.results.title')}</h2>
		<div class="result-slots">
			{#each [0,1,2,3,4] as i}
				{@const file = $pptResults[i]}
				<button
					class="slot-btn"
					class:slot-filled={!!file}
					onclick={() => file && handleOpenSlot(file.path)}
					disabled={!file || loading}
				>
					{file ? file.name : `— ${i + 1} —`}
				</button>
			{/each}
		</div>
	</section>

	<!-- Presentation Status -->
	<section class="section">
		<h2>{$_('presentations.status.title')}</h2>
		<div class="status-grid">
			<div class="status-item">
				<span class="status-label">{$_('presentations.status.slideshow')}</span>
				<span class="status-value" class:active={$keynoteStatus.slideshowActive}>
					{$keynoteStatus.slideshowActive ? $_('presentations.status.on') : $_('presentations.status.off')}
				</span>
			</div>
			<div class="status-item">
				<span class="status-label">{$_('presentations.status.slide')}</span>
				<span class="status-value">
					{$keynoteStatus.currentSlide ?? '—'} / {$keynoteStatus.totalSlides ?? '—'}
				</span>
			</div>
			<div class="status-item">
				<span class="status-label">{$_('presentations.status.document')}</span>
				<span class="status-value">{$keynoteStatus.documentName ?? '—'}</span>
			</div>
		</div>
	</section>

	<!-- Control Bar -->
	<section class="section">
		<h2>{$_('presentations.controls.title')}</h2>
		<div class="control-bar">
			<button class="ctrl-btn" onclick={handleFirst} aria-label={$_('presentations.controls.first')}>⏮</button>
			<button class="ctrl-btn" onclick={handlePrev} aria-label={$_('presentations.controls.prev')}>◀</button>
			<button class="ctrl-btn" onclick={handleNext} aria-label={$_('presentations.controls.next')}>▶</button>
			<button class="ctrl-btn" onclick={handleLast} aria-label={$_('presentations.controls.last')}>⏭</button>
			<button class="ctrl-btn play-btn" onclick={handleStart} aria-label={$_('presentations.controls.play')}>▶ {$_('presentations.controls.play')}</button>
			<button class="ctrl-btn stop-btn" onclick={handleStop} aria-label={$_('presentations.controls.stop')}>⏹ {$_('presentations.controls.stop')}</button>
			<button class="ctrl-btn close-btn" onclick={handleCloseAll} aria-label={$_('presentations.controls.closeAll')}>✕ {$_('presentations.controls.closeAll')}</button>
		</div>
	</section>
</div>

<style>
	.presentations-page {
		padding: 1rem 0;
	}

	h1 {
		font-size: 1.5rem;
		font-weight: 600;
		margin-bottom: 1.5rem;
	}

	.section {
		margin-bottom: 2rem;
	}

	h2 {
		font-size: 1.1rem;
		font-weight: 600;
		margin-bottom: 0.75rem;
		color: #374151;
	}

	.folder-list {
		list-style: none;
		padding: 0;
		margin: 0 0 0.75rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.folder-item {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.5rem 0.75rem;
		background: #f3f4f6;
		border-radius: 0.375rem;
	}

	.folder-name {
		font-weight: 500;
		min-width: 120px;
	}

	.folder-path {
		flex: 1;
		color: #6b7280;
		font-size: 0.875rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.folder-item button {
		padding: 0.25rem 0.5rem;
		background: #ef4444;
		color: white;
		border: none;
		border-radius: 0.25rem;
		cursor: pointer;
		font-size: 0.75rem;
	}

	.btn-primary {
		padding: 0.5rem 1rem;
		background: #3b82f6;
		color: white;
		border: none;
		border-radius: 0.375rem;
		cursor: pointer;
		font-size: 0.875rem;
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.filter-display {
		padding: 0.5rem 0.75rem;
		background: #111827;
		color: #f9fafb;
		border-radius: 0.375rem;
		font-size: 1.5rem;
		font-family: monospace;
		min-height: 2.5rem;
		margin-bottom: 0.75rem;
		letter-spacing: 0.1em;
	}

	.digit-pad {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 0.5rem;
		max-width: 280px;
	}

	.digit-btn {
		padding: 0.75rem;
		font-size: 1.25rem;
		font-weight: 600;
		background: #3b82f6;
		color: white;
		border: none;
		border-radius: 0.375rem;
		cursor: pointer;
	}

	.backspace-btn {
		background: #ef4444;
	}

	.clear-btn {
		background: #f59e0b;
	}

	.result-slots {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.slot-btn {
		padding: 0.75rem 1rem;
		background: #6b7280;
		color: white;
		border: none;
		border-radius: 0.375rem;
		cursor: pointer;
		text-align: left;
		font-size: 0.875rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.slot-btn.slot-filled {
		background: #059669;
	}

	.slot-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.status-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 1rem;
	}

	.status-item {
		padding: 0.75rem;
		background: #f3f4f6;
		border-radius: 0.375rem;
	}

	.status-label {
		display: block;
		font-size: 0.75rem;
		color: #6b7280;
		margin-bottom: 0.25rem;
	}

	.status-value {
		font-weight: 600;
		font-size: 0.875rem;
	}

	.status-value.active {
		color: #059669;
	}

	.control-bar {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap;
	}

	.ctrl-btn {
		padding: 0.5rem 0.75rem;
		background: #3b82f6;
		color: white;
		border: none;
		border-radius: 0.375rem;
		cursor: pointer;
		font-size: 0.875rem;
	}

	.play-btn {
		background: #059669;
	}

	.stop-btn {
		background: #9a3412;
	}

	.close-btn {
		background: #7f1d1d;
	}
</style>
