<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { onMount } from 'svelte';
	import Card from '$lib/components/ui/card.svelte';
	import CardHeader from '$lib/components/ui/card-header.svelte';
	import CardTitle from '$lib/components/ui/card-title.svelte';
	import CardDescription from '$lib/components/ui/card-description.svelte';
	import CardContent from '$lib/components/ui/card-content.svelte';
	import Label from '$lib/components/ui/label.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import Checkbox from '$lib/components/ui/checkbox.svelte';
	import Select from '$lib/components/ui/select.svelte';
	import SelectTrigger from '$lib/components/ui/select-trigger.svelte';
	import SelectValue from '$lib/components/ui/select-value.svelte';
	import SelectContent from '$lib/components/ui/select-content.svelte';
	import SelectItem from '$lib/components/ui/select-item.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import Separator from '$lib/components/ui/separator.svelte';
	import {
		uploadSettingsStore,
		uploadSettings,
		youtubeConfig
	} from '$lib/stores/upload-settings-store';
	import type { UploadSettings, YouTubeUploadConfig } from '$lib/types/upload-config';
	import { toast } from '$lib/utils/toast';
	import { Upload, Youtube, Facebook, Settings2 } from 'lucide-svelte';

	// Local state for form
	let minDurationMinutes = 45;
	let shortVideoThresholdMinutes = 10;
	let retryAttempts = 3;
	let chunkSizeMB = 10;

	// YouTube settings
	let youtubeEnabled = true;
	let youtubeAutoUpload = true;
	let youtubePublishAfterUpload = true;
	let youtubeDefaultPrivacy: 'public' | 'unlisted' | 'private' = 'public';

	// Initialize settings on mount
	onMount(async () => {
		await uploadSettingsStore.init();
		loadSettings();
	});

	// Load settings into local state
	function loadSettings() {
		const settings = uploadSettingsStore.getSettings();
		minDurationMinutes = settings.minDurationMinutes;
		shortVideoThresholdMinutes = settings.shortVideoThresholdMinutes;
		retryAttempts = settings.retryAttempts;
		chunkSizeMB = settings.chunkSizeMB;

		const ytConfig = uploadSettingsStore.getPlatformConfig<YouTubeUploadConfig>('youtube');
		if (ytConfig) {
			youtubeEnabled = ytConfig.enabled;
			youtubeAutoUpload = ytConfig.autoUpload;
			youtubePublishAfterUpload = ytConfig.publishAfterUpload;
			youtubeDefaultPrivacy = ytConfig.defaultPrivacy;
		}
	}

	// Save global settings
	async function saveGlobalSettings() {
		await uploadSettingsStore.updateSettings({
			minDurationMinutes,
			shortVideoThresholdMinutes,
			retryAttempts,
			chunkSizeMB
		});

		toast({
			title: $_('settings.upload.saved') || 'Settings saved',
			variant: 'success'
		});
	}

	// Save YouTube settings
	async function saveYoutubeSettings() {
		await uploadSettingsStore.setPlatformConfig({
			platform: 'youtube',
			enabled: youtubeEnabled,
			autoUpload: youtubeAutoUpload,
			useEventPrivacy: true,
			defaultPrivacy: youtubeDefaultPrivacy,
			publishAfterUpload: youtubePublishAfterUpload
		});

		toast({
			title: $_('settings.upload.youtube.saved') || 'YouTube settings saved',
			variant: 'success'
		});
	}

	// Handle checkbox changes
	function handleYoutubeEnabledChange(event: Event) {
		const target = event.target as HTMLInputElement;
		youtubeEnabled = target.checked;
		saveYoutubeSettings();
	}

	function handleYoutubeAutoUploadChange(event: Event) {
		const target = event.target as HTMLInputElement;
		youtubeAutoUpload = target.checked;
		saveYoutubeSettings();
	}

	function handleYoutubePublishChange(event: Event) {
		const target = event.target as HTMLInputElement;
		youtubePublishAfterUpload = target.checked;
		saveYoutubeSettings();
	}

	// Handle input blur to save
	function handleGlobalInputBlur() {
		saveGlobalSettings();
	}
</script>

<Card>
	<CardHeader>
		<CardTitle className="flex items-center gap-2">
			<Upload class="h-5 w-5" />
			{$_('settings.upload.title') || 'Upload Settings'}
		</CardTitle>
		<CardDescription>
			{$_('settings.upload.description') || 'Configure automatic upload to video platforms after events'}
		</CardDescription>
	</CardHeader>

	<CardContent className="space-y-6">
		<!-- Global Settings -->
		<div class="space-y-4">
			<h4 class="text-sm font-medium flex items-center gap-2">
				<Settings2 class="h-4 w-4" />
				{$_('settings.upload.global') || 'Global Settings'}
			</h4>

			<div class="grid gap-4 sm:grid-cols-2">
				<div class="space-y-2">
					<Label htmlFor="minDuration">
						{$_('settings.upload.minDuration') || 'Minimum duration (minutes)'}
					</Label>
					<Input
						id="minDuration"
						type="number"
						bind:value={minDurationMinutes}
						min="1"
						max="180"
						onblur={handleGlobalInputBlur}
					/>
					<p class="text-xs text-muted-foreground">
						{$_('settings.upload.minDurationHint') || "Events shorter than this won't trigger auto-upload"}
					</p>
				</div>

				<div class="space-y-2">
					<Label htmlFor="shortThreshold">
						{$_('settings.upload.shortThreshold') || 'Short video threshold (minutes)'}
					</Label>
					<Input
						id="shortThreshold"
						type="number"
						bind:value={shortVideoThresholdMinutes}
						min="1"
						max="60"
						onblur={handleGlobalInputBlur}
					/>
					<p class="text-xs text-muted-foreground">
						{$_('settings.upload.shortThresholdHint') || 'Videos shorter than this are ignored when selecting'}
					</p>
				</div>

				<div class="space-y-2">
					<Label htmlFor="retryAttempts">
						{$_('settings.upload.retryAttempts') || 'Retry attempts'}
					</Label>
					<Input
						id="retryAttempts"
						type="number"
						bind:value={retryAttempts}
						min="0"
						max="10"
						onblur={handleGlobalInputBlur}
					/>
				</div>

				<div class="space-y-2">
					<Label htmlFor="chunkSize">
						{$_('settings.upload.chunkSize') || 'Chunk size (MB)'}
					</Label>
					<Input
						id="chunkSize"
						type="number"
						bind:value={chunkSizeMB}
						min="1"
						max="50"
						onblur={handleGlobalInputBlur}
					/>
				</div>
			</div>
		</div>

		<Separator />

		<!-- YouTube Settings -->
		<div class="space-y-4">
			<div class="flex items-center justify-between">
				<h4 class="text-sm font-medium flex items-center gap-2">
					<Youtube class="h-4 w-4 text-red-600" />
					YouTube
				</h4>
				<Checkbox
					id="youtubeEnabled"
					checked={youtubeEnabled}
					onchange={handleYoutubeEnabledChange}
				/>
			</div>

			{#if youtubeEnabled}
				<div class="space-y-3 pl-6">
					<div class="flex items-center gap-2">
						<Checkbox
							id="youtubeAutoUpload"
							checked={youtubeAutoUpload}
							onchange={handleYoutubeAutoUploadChange}
						/>
						<Label htmlFor="youtubeAutoUpload" className="text-sm font-normal cursor-pointer">
							{$_('settings.upload.youtube.autoUpload') || 'Auto-upload recording after event'}
						</Label>
					</div>

					<div class="flex items-center gap-2">
						<Checkbox
							id="youtubePublish"
							checked={youtubePublishAfterUpload}
							onchange={handleYoutubePublishChange}
						/>
						<Label htmlFor="youtubePublish" className="text-sm font-normal cursor-pointer">
							{$_('settings.upload.youtube.autoPublish') || 'Publish video automatically when ready'}
						</Label>
					</div>

					<div class="space-y-2">
						<Label htmlFor="youtubePrivacy">
							{$_('settings.upload.youtube.defaultPrivacy') || 'Default privacy'}
						</Label>
						<select
							id="youtubePrivacy"
							class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
							bind:value={youtubeDefaultPrivacy}
							onchange={saveYoutubeSettings}
						>
							<option value="public">{$_('common.public') || 'Public'}</option>
							<option value="unlisted">{$_('common.unlisted') || 'Unlisted'}</option>
							<option value="private">{$_('common.private') || 'Private'}</option>
						</select>
					</div>
				</div>
			{/if}
		</div>

		<Separator />

		<!-- Facebook Settings (Coming Soon) -->
		<div class="space-y-4 opacity-50">
			<div class="flex items-center justify-between">
				<h4 class="text-sm font-medium flex items-center gap-2">
					<Facebook class="h-4 w-4 text-blue-600" />
					Facebook
				</h4>
				<Badge variant="secondary">{$_('common.comingSoon') || 'Coming soon'}</Badge>
			</div>
		</div>
	</CardContent>
</Card>
