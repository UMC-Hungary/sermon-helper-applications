import { toast as sonnerToast } from 'svelte-sonner';

interface ToastOptions {
	title?: string;
	description?: string;
}

export function toast({ title, description }: ToastOptions) {
	sonnerToast(title || '', {
		description,
	});
}
