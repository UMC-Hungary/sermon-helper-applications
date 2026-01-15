<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { youtubeApi } from '$lib/utils/youtube-api';
	import { toast } from '$lib/utils/toast';
	import { Loader2 } from 'lucide-svelte';

	let status = 'Processing login...';
	let error = '';

	onMount(async () => {
		const code = $page.url.searchParams.get('code');
		const errorParam = $page.url.searchParams.get('error');

		if (errorParam) {
			error = $page.url.searchParams.get('error_description') || errorParam;
			toast({
				title: 'Login Failed',
				description: error,
				variant: 'error'
			});
			setTimeout(() => goto('/'), 2000);
			return;
		}

		if (!code) {
			error = 'No authorization code received';
			toast({
				title: 'Login Failed',
				description: error,
				variant: 'error'
			});
			setTimeout(() => goto('/'), 2000);
			return;
		}

		try {
			status = 'Exchanging authorization code...';
			await youtubeApi.exchangeCodeForTokens(code);

			toast({
				title: 'Logged In',
				description: 'Successfully logged in to YouTube',
				variant: 'success'
			});

			// Redirect to home or events page
			goto('/');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to complete login';
			toast({
				title: 'Login Failed',
				description: error,
				variant: 'error'
			});
			setTimeout(() => goto('/'), 3000);
		}
	});
</script>

<div class="flex flex-col items-center justify-center min-h-[50vh] space-y-4">
	{#if error}
		<div class="text-center space-y-2">
			<h2 class="text-xl font-semibold text-destructive">Login Failed</h2>
			<p class="text-muted-foreground">{error}</p>
			<p class="text-sm text-muted-foreground">Redirecting...</p>
		</div>
	{:else}
		<Loader2 class="h-8 w-8 animate-spin text-primary" />
		<p class="text-muted-foreground">{status}</p>
	{/if}
</div>
