<script lang="ts">
	import '../app.css';
    import { Toaster } from 'svelte-sonner';
    import ErrorMessages from "$lib/components/ui/error-messages.svelte";
    import Sidebar from '$lib/components/sidebar.svelte';
    import '$lib/i18n'; // Initialize i18n at module level
    import { loadSavedLocale } from '$lib/i18n';
    import { isLoading } from 'svelte-i18n';
    import { onMount } from 'svelte';
    import { obsWebSocket } from "$lib/utils/obs-websocket";
    import { appSettingsStore, appSettingsLoaded } from '$lib/utils/app-settings-store';

    let { children } = $props();

    onMount(async () => {
        // Load app settings first (before rendering pages)
        await appSettingsStore.load();

        loadSavedLocale();
        obsWebSocket.autoconnect();
    });

    let isMobileMenuOpen = false;

    let textus = 'John 3:16-21';
    let leckio = 'Romans 8:28-39';

    let currentSermon = {
        youtubeTitle: 'The Love of God - Sunday Service',
        youtubeScheduled: true,
        streamStarted: false,
    };

    let youtubeOnAir = false;

    const handleSystemRecheck = async () => {
        console.log('[v0] Rechecking system status...');
        await new Promise((resolve) => setTimeout(resolve, 500));
    };

    const handleYoutubeLogin = () => {
        // systemStatus = { ...systemStatus, youtubeLoggedIn: true };
    };

    let isRechecking = false;

    function handleRecheck(): void {
        isRechecking = true;
        // Simulate recheck logic
        setTimeout(() => {
            isRechecking = false;
        }, 2000);
    }

    // Event handler
    let onRecheck: () => Promise<void> = async () => {};
</script>

<Toaster 
		position="bottom-right" 
		toastOptions={{
			classes: {
				success: 'bg-white text-gray-800 border-2 border-green-600',
				error: 'bg-white text-gray-800 border-2 border-red-600',  
				warning: 'bg-white text-gray-800 border-2 border-yellow-500',
				info: 'bg-white text-gray-800 border-2 border-blue-600',
			},
		}}
	/>

{#if $isLoading || !$appSettingsLoaded}
<div class="flex h-screen items-center justify-center bg-background">
    <div class="animate-pulse text-muted-foreground">Loading...</div>
</div>
{:else}
<div class="flex h-screen overflow-hidden bg-background">
    <Sidebar
            isMobileMenuOpen={isMobileMenuOpen}
            onMobileMenuToggle={() => isMobileMenuOpen = !isMobileMenuOpen}
            currentSermon={{ textus, leckio, ...currentSermon }}
    />

    <main class="flex-1 overflow-y-auto">
        <div class="p-4 lg:p-8 space-y-6 pt-20 lg:pt-8">
            <ErrorMessages onRecheck={onRecheck} />

            {@render children()}
        </div>
    </main>
</div>
{/if}