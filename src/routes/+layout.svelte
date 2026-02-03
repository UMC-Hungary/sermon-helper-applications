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
    import { sessionIntegration } from '$lib/services/session-integration';
    import UpdateChecker from '$lib/components/update-checker.svelte';
    import { browser } from '$app/environment';
    import { isTauriApp } from '$lib/utils/storage-helpers';
    import { discoveryServerManager, discoveryServerStatus } from '$lib/stores/discovery-server-store';
    import { systemStore, obsStatus } from '$lib/stores/system-store';
    import type { DiscoverySystemStatus, DiscoveryObsStatus } from '$lib/types/discovery';

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

            // Only initialize session integration in Tauri desktop app
            if (isTauriApp()) {
                await sessionIntegration.init();
                await log('info', 'Session integration initialized');

                // Initialize discovery server manager
                await discoveryServerManager.init();
                await log('info', 'Discovery server manager initialized');

                // Auto-start discovery server if enabled and has auth token
                const discoverySettings = await appSettingsStore.get('discoverySettings');
                await log('info', `Discovery settings loaded: autoStart=${discoverySettings?.autoStart}, hasToken=${!!discoverySettings?.authToken}`);
                if (discoverySettings?.autoStart && discoverySettings?.authToken) {
                    try {
                        await discoveryServerManager.start(discoverySettings);
                        await log('info', 'Discovery server auto-started');

                        // Sync RF/IR commands to the server
                        const { broadlinkService } = await import('$lib/utils/broadlink-service');
                        await broadlinkService.syncCommandsToServer();
                        await log('info', 'RF/IR commands synced to discovery server');
                    } catch (e) {
                        await log('warn', `Failed to auto-start discovery server: ${e}`);
                    }
                }
            } else {
                await log('info', 'Session integration skipped (web mode)');
            }

            await log('info', 'Layout onMount completed successfully');
        } catch (e) {
            await log('error', `Layout onMount error: ${e}`);
        }
    });

    // Subscriptions for broadcasting status to discovery server
    let systemUnsubscribe: (() => void) | undefined;
    let obsUnsubscribe: (() => void) | undefined;

    // Set up subscriptions after mount
    $effect(() => {
        if (browser && isTauriApp() && $discoveryServerStatus.running) {
            // Subscribe to system store changes
            systemUnsubscribe = systemStore.subscribe(($system) => {
                const status: DiscoverySystemStatus = {
                    obsConnected: $system.obs,
                    obsStreaming: false, // Will be updated by OBS status
                    obsRecording: false,
                    rodeInterface: true,
                    mainDisplay: true,
                    secondaryDisplay: true,
                    youtubeLoggedIn: $system.youtubeLoggedIn
                };
                discoveryServerManager.updateSystemStatus(status);
            });

            // Subscribe to OBS status changes
            obsUnsubscribe = obsStatus.subscribe(($obs) => {
                const status: DiscoveryObsStatus = {
                    connected: $obs.connected,
                    streaming: false, // TODO: Add streaming status to obsStatus
                    recording: false, // TODO: Add recording status to obsStatus
                    streamTimecode: null,
                    recordTimecode: null
                };
                discoveryServerManager.updateObsStatus(status);
            });
        }

        return () => {
            systemUnsubscribe?.();
            obsUnsubscribe?.();
        };
    });

    onDestroy(() => {
        refreshStore.stop();
        systemUnsubscribe?.();
        obsUnsubscribe?.();
    });

    let isMobileMenuOpen = $state(false);

    function toggleMobileMenu() {
        isMobileMenuOpen = !isMobileMenuOpen;
    }

    const handleRecheck = async () => {
        await handleReconnect();
        console.log("Rechecking all systems...");
    };

    const handleReconnect = async () => {
        await obsWebSocket.autoconnect();
    };
</script>

<Toaster
        position="top-right" richColors closeButton
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
             <ErrorMessages onRecheck={handleRecheck} onReconnect={handleReconnect} />

             {@render children()}
         </div>
     </main>
</div>
{/if}