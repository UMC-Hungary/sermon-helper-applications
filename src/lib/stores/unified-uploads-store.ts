// Unified Uploads Store
// Combines active session uploads with pending uploads from events

import { derived } from 'svelte/store';
import { currentSession, uploadProgress } from '$lib/stores/event-session-store';
import { allPendingUploads, eventList } from '$lib/stores/event-store';
import { generateCalculatedTitle, type ServiceEvent, type EventUploadSession } from '$lib/types/event';
import type { UploadPlatform } from '$lib/types/upload-config';
import type { PlatformUploadProgress } from '$lib/types/event-session';

// Unified upload item for display
export interface UnifiedUploadItem {
	id: string;
	eventId: string;
	event: ServiceEvent;
	calculatedTitle: string;
	platform: UploadPlatform;
	status: 'uploading' | 'processing' | 'pending' | 'paused' | 'failed';
	progress?: {
		bytesUploaded: number;
		totalBytes: number;
		percentage: number;
	};
	error?: string;
	startedAt: number;
	source: 'session' | 'event';
}

// Convert session upload progress to unified item
function sessionProgressToUnified(
	progress: PlatformUploadProgress,
	session: { eventId: string },
	event: ServiceEvent | undefined
): UnifiedUploadItem | null {
	if (!event) return null;

	// Only include active statuses
	if (progress.status === 'completed') return null;

	return {
		id: `session-${progress.platform}-${session.eventId}`,
		eventId: session.eventId,
		event,
		calculatedTitle: generateCalculatedTitle(event),
		platform: progress.platform,
		status: progress.status === 'initializing' ? 'pending' : progress.status as UnifiedUploadItem['status'],
		progress: progress.status === 'uploading' ? {
			bytesUploaded: progress.bytesUploaded,
			totalBytes: progress.totalBytes,
			percentage: progress.percentage
		} : undefined,
		error: progress.error,
		startedAt: Date.now(), // Session uploads don't have startedAt
		source: 'session'
	};
}

// Convert event upload session to unified item
function eventSessionToUnified(
	session: EventUploadSession,
	event: ServiceEvent
): UnifiedUploadItem | null {
	// Only include non-completed statuses
	if (session.status === 'completed') return null;

	return {
		id: session.id,
		eventId: event.id,
		event,
		calculatedTitle: generateCalculatedTitle(event),
		platform: session.platform,
		status: session.status as UnifiedUploadItem['status'],
		progress: session.status === 'uploading' ? {
			bytesUploaded: session.bytesUploaded,
			totalBytes: session.fileSize,
			percentage: session.fileSize > 0 ? (session.bytesUploaded / session.fileSize) * 100 : 0
		} : undefined,
		error: session.error,
		startedAt: session.startedAt,
		source: 'event'
	};
}

// Main unified uploads store
export const unifiedUploads = derived(
	[currentSession, uploadProgress, allPendingUploads, eventList],
	([$session, $progress, $pendingUploads, $events]) => {
		const items: UnifiedUploadItem[] = [];
		const seenIds = new Set<string>();

		// 1. Add active session uploads (highest priority)
		if ($session && $progress.length > 0) {
			const sessionEvent = $events.find(e => e.id === $session.eventId);

			for (const progress of $progress) {
				const item = sessionProgressToUnified(progress, $session, sessionEvent);
				if (item) {
					items.push(item);
					// Mark this event+platform as seen to avoid duplicates
					seenIds.add(`${item.eventId}-${item.platform}`);
				}
			}
		}

		// 2. Add pending uploads from events (avoid duplicates)
		for (const { event, session } of $pendingUploads) {
			const key = `${event.id}-${session.platform}`;
			if (seenIds.has(key)) continue;

			const item = eventSessionToUnified(session, event);
			if (item) {
				items.push(item);
				seenIds.add(key);
			}
		}

		// Sort: uploading first, then by startedAt (most recent first)
		return items.sort((a, b) => {
			// Uploading status first
			if (a.status === 'uploading' && b.status !== 'uploading') return -1;
			if (b.status === 'uploading' && a.status !== 'uploading') return 1;

			// Then by startedAt (most recent first)
			return b.startedAt - a.startedAt;
		});
	}
);

// Derived convenience stores
export const hasUploads = derived(unifiedUploads, ($uploads) => $uploads.length > 0);
export const uploadCount = derived(unifiedUploads, ($uploads) => $uploads.length);
export const hasActiveUpload = derived(unifiedUploads, ($uploads) =>
	$uploads.some(u => u.status === 'uploading')
);
export const pendingUploadCount = derived(unifiedUploads, ($uploads) =>
	$uploads.filter(u => u.status !== 'uploading').length
);
