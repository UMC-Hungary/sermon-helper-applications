export interface Event {
  id: string;
  title: string;
  dateTime: string;
  speaker: string;
  description: string;
  textus: string;
  leckio: string;
  textusTranslation: string;
  leckioTranslation: string;
  youtubePrivacyStatus: 'public' | 'private' | 'unlisted';
  autoUploadEnabled: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface EventSummary {
  id: string;
  title: string;
  dateTime: string;
  speaker: string;
  recordingCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface CreateEventPayload {
  title: string;
  date_time: string;
  speaker?: string;
  description?: string;
  textus?: string;
  leckio?: string;
  textus_translation?: string;
  leckio_translation?: string;
  youtube_privacy_status?: string;
  auto_upload_enabled?: boolean;
}
