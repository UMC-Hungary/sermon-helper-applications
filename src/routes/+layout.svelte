<script lang="ts">
	import '../app.css';
    import { Toaster } from 'svelte-sonner';
    import ErrorMessages from "$lib/components/ui/error-messages.svelte";
    import Sidebar from '$lib/components/sidebar.svelte';
    import '$lib/i18n'; // Initialize i18n at module level
    import { loadSavedLocale } from '$lib/i18n';
    import { isLoading } from 'svelte-i18n';
    import { onMount, onDestroy } from 'svelte';
    import { obsWebSocket } from "$lib/utils/obs-websocket";
    import { appSettingsStore, appSettingsLoaded } from '$lib/utils/app-settings-store';
    import { initTheme } from '$lib/stores/theme-store';
    import { initOAuthHandler } from '$lib/utils/oauth-handler';
    import { youtubeAuthStore } from '$lib/stores/youtube-store';
    import { updateYoutubeLogin } from '$lib/stores/system-store';
    import { refreshStore } from '$lib/stores/refresh-store';
    import UpdateChecker from '$lib/components/update-checker.svelte';
    import { browser } from '$app/environment';

    let { children } = $props();

    // Logging helper that works in both browser and Tauri
    async function log(level: 'info' | 'error' | 'warn' | 'debug', message: string) {
        console.log(`[${level.toUpperCase()}] ${message}`);
        if (browser && '__TAURI_INTERNALS__' in window) {
            try {
                const { info, error, warn, debug } = await import('@tauri-apps/plugin-log');
                const logFn = { info, error, warn, debug }[level];
                await logFn(message);
            } catch (e) {
                console.error('Failed to log to Tauri:', e);
            }
        }
    }

    onMount(async () => {
        await log('info', 'Layout onMount started');
        try {
            await log('info', 'Initializing theme...');
            await initTheme();
            await log('info', 'Theme initialized');

            await log('info', 'Loading app settings...');
            await appSettingsStore.load();
            await log('info', 'App settings loaded');

            await log('info', 'Initializing OAuth handler...');
            await initOAuthHandler();
            await log('info', 'OAuth handler initialized');

            updateYoutubeLogin(youtubeAuthStore.isLoggedIn());
            await log('info', 'YouTube login status updated');

            loadSavedLocale();
            await log('info', 'Locale loaded');

            obsWebSocket.autoconnect();
            await log('info', 'OBS autoconnect started');

            refreshStore.start();
            await log('info', 'Refresh store started');

            await log('info', 'Layout onMount completed successfully');
        } catch (e) {
            await log('error', `Layout onMount error: ${e}`);
        }
    });

    onDestroy(() => {
        refreshStore.stop();
    });

    let isMobileMenuOpen = $state(false);

    function toggleMobileMenu() {
        isMobileMenuOpen = !isMobileMenuOpen;
    }

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
<UpdateChecker />

{#if $isLoading || !$appSettingsLoaded}
<div class="flex h-screen items-center justify-center bg-background">
    <div class="animate-pulse text-muted-foreground">Loading...</div>
</div>
{:else}
<div class="flex h-screen overflow-hidden bg-background">
    <Sidebar
            isMobileMenuOpen={isMobileMenuOpen}
            onMobileMenuToggle={toggleMobileMenu}
    />

     <main class="flex-1 overflow-y-auto">
         <div class="p-4 md:p-8 space-y-6 pt-20 md:pt-8">
             <ErrorMessages onRecheck={onRecheck} />

             {@render children()}
         </div>
     </main>
</div>
{/if}