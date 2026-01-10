import { toast as sonnerToast } from 'svelte-sonner';

type ToastVariant = 'success' | 'error' | 'info' | 'warning';

interface ToastOptions {
	title?: string;
	description?: string;
	variant?: ToastVariant;
	duration?: number;
}

export function toast({ title, description, variant = 'info', duration }: ToastOptions) {
	if (variant === 'success') {
		sonnerToast.success(title || '', {
			style: 'border: 2px solid green !important;',
			description,
			duration,
		});
	} else if (variant === 'error') {
		sonnerToast.error(title || '', {
			style: 'border: 2px solid red !important;',
			description,
			duration,
		});
	} else if (variant === 'warning') {
		sonnerToast.warning(title || '', {
			description,
			duration,
		});
	} else {
		sonnerToast.info(title || '', {
			description,
			duration,
		});
	}
}
