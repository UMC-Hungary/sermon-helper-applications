<script lang="ts">
	import '../app.css';
	import { Toaster } from 'svelte-sonner';
    import ErrorMessages from "$lib/components/ui/error-messages.svelte";
    import Sidebar from '$lib/components/sidebar.svelte';

    let { children } = $props();

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
			unstyled: true,
			classes: {
				success: 'bg-green-600 text-white border border-green-700 rounded-md shadow-lg p-4 mb-2',
				error: 'bg-red-600 text-white border border-red-700 rounded-md shadow-lg p-4 mb-2',
				warning: 'bg-yellow-500 text-black border border-yellow-600 rounded-md shadow-lg p-4 mb-2',
				info: 'bg-blue-600 text-white border border-blue-700 rounded-md shadow-lg p-4 mb-2',
			},
		}}
	/>

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