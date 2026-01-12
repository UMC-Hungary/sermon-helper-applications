<script lang="ts">
	import { Calendar, AlertCircle, LogIn } from 'lucide-svelte';
	import { toast } from 'svelte-sonner';
	import { _ } from 'svelte-i18n';
	import { systemStore, updateYoutubeLogin } from '$lib/stores/system-store';
	import { goto } from '$app/navigation';
	import Button from '$lib/components/ui/button.svelte';
	import Input from '$lib/components/ui/input.svelte';
	import Label from '$lib/components/ui/label.svelte';
	import Textarea from '$lib/components/ui/textarea.svelte';
	import Alert from '$lib/components/ui/alert.svelte';
	import AlertTitle from '$lib/components/ui/alert-title.svelte';
	import AlertDescription from '$lib/components/ui/alert-description.svelte';

	let title = $state('');
	let speaker = $state('');
	let date = $state('');
	let time = $state('');
	let privacy = $state<'public' | 'private' | 'unlisted'>('public');
	let description = $state('');

	const systemStatus = $derived($systemStore);
	const youtubeLoggedIn = $derived(systemStatus?.youtubeLoggedIn ?? false);

	function handleSchedule() {
		toast.success($_('toasts.eventScheduled.title'), {
			description: $_('toasts.eventScheduled.description')
		});

		// Reset form
		title = '';
		speaker = '';
		date = '';
		time = '';
		description = '';
	}

	function handleGoogleLogin() {
		toast.success($_('toasts.loggedIn.title'), {
			description: $_('toasts.loggedIn.description')
		});
		
		// Update system status to show logged in
		updateYoutubeLogin(true);

		// Navigate back after a short delay to show the toast
		setTimeout(() => {
			goto('/');
		}, 500);
	}

	function handleRecheck() {
		// Recheck system status - in a real app, this would check actual system status
		console.log('Rechecking system status...');
	}
</script>

<div class="p-4 lg:p-8 space-y-6 pt-20 lg:pt-8">
	<div>
		<h2 class="text-3xl font-bold tracking-tight">{$_('youtubeSchedule.title')}</h2>
		<p class="text-muted-foreground">{$_('youtubeSchedule.subtitle')}</p>
	</div>

	{#if !youtubeLoggedIn}
		<div class="rounded-lg border bg-card text-card-foreground shadow-sm p-6">
			<Alert>
				<AlertCircle class="h-4 w-4" />
				<AlertTitle>{$_('youtubeSchedule.loginRequired.title')}</AlertTitle>
				<AlertDescription class="space-y-4">
					<p>{$_('youtubeSchedule.loginRequired.description')}</p>
					<Button onclick={handleGoogleLogin} class="w-full sm:w-auto">
						<LogIn class="mr-2 h-4 w-4" />
						{$_('youtubeSchedule.loginRequired.loginButton')}
					</Button>
				</AlertDescription>
			</Alert>
		</div>
	{:else}
		<div class="rounded-lg border bg-card text-card-foreground shadow-sm">
			<div class="flex flex-col space-y-1.5 p-6">
				<h3 class="text-2xl font-semibold leading-none tracking-tight">{$_('youtubeSchedule.eventDetails.title')}</h3>
				<p class="text-sm text-muted-foreground">{$_('youtubeSchedule.eventDetails.description')}</p>
			</div>
			<div class="p-6 pt-0 space-y-6">
				<div class="space-y-2">
					<Label for="title">{$_('youtubeSchedule.eventDetails.eventTitle.label')}</Label>
					<Input
						id="title"
						bind:value={title}
						placeholder={$_('youtubeSchedule.eventDetails.eventTitle.placeholder')}
					/>
				</div>

				<div class="space-y-2">
					<Label for="speaker">{$_('youtubeSchedule.eventDetails.speaker.label')}</Label>
					<Input
						id="speaker"
						bind:value={speaker}
						placeholder={$_('youtubeSchedule.eventDetails.speaker.placeholder')}
					/>
				</div>

				<div class="grid gap-4 md:grid-cols-2">
					<div class="space-y-2">
						<Label for="date">{$_('youtubeSchedule.eventDetails.date.label')}</Label>
						<Input id="date" type="date" bind:value={date} />
					</div>

					<div class="space-y-2">
						<Label for="time">{$_('youtubeSchedule.eventDetails.time.label')}</Label>
						<Input id="time" type="time" bind:value={time} />
					</div>
				</div>

				<div class="space-y-2">
					<Label for="privacy">{$_('youtubeSchedule.eventDetails.privacy.label')}</Label>
					<select 
						id="privacy"
						bind:value={privacy}
						class="flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
					>
						<option value="public">{$_('youtubeSchedule.eventDetails.privacy.options.public')}</option>
						<option value="unlisted">{$_('youtubeSchedule.eventDetails.privacy.options.unlisted')}</option>
						<option value="private">{$_('youtubeSchedule.eventDetails.privacy.options.private')}</option>
					</select>
				</div>

				<div class="space-y-2">
					<Label for="description">{$_('youtubeSchedule.eventDetails.description.label')}</Label>
					<Textarea
						id="description"
						bind:value={description}
						placeholder={$_('youtubeSchedule.eventDetails.description.placeholder')}
						rows={6}
						class="resize-none"
					/>
					<p class="text-xs text-muted-foreground">
						{$_('youtubeSchedule.eventDetails.description.tip')}
					</p>
				</div>

				<Button class="w-full" onclick={handleSchedule}>
					<Calendar class="mr-2 h-4 w-4" />
					{$_('youtubeSchedule.eventDetails.scheduleButton')}
				</Button>
			</div>
		</div>
	{/if}
</div>