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
			description,
			duration,
		});
	} else if (variant === 'error') {
		sonnerToast.error(title || '', {
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

// Convenience methods for specific variants
export const toastSuccess = (title: string, description?: string, duration?: number) => {
	toast({ title, description, variant: 'success', duration });
};

export const toastError = (title: string, description?: string, duration?: number) => {
	toast({ title, description, variant: 'error', duration });
};

export const toastWarning = (title: string, description?: string, duration?: number) => {
	toast({ title, description, variant: 'warning', duration });
};

export const toastInfo = (title: string, description?: string, duration?: number) => {
	toast({ title, description, variant: 'info', duration });
};
