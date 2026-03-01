<script lang="ts">
	import { errorCount } from '$lib/stores/errors.js';
	import { _ } from 'svelte-i18n';
</script>

{#if $errorCount > 0}
	<a href="/errors" class="error-badge">
		{$_('errorsPage.title')}: {$errorCount}
	</a>
{/if}

<style>
	.error-badge {
		display: inline-flex;
		align-items: center;
		padding: 0.2rem 0.6rem;
		border-radius: 9999px;
		font-size: 0.8rem;
		font-weight: 600;
		background: #fee2e2;
		color: #991b1b;
		text-decoration: none;
	}

	.error-badge:hover {
		background: #fecaca;
	}
</style>
