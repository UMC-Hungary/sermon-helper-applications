export interface Recording {
  id: string;
  eventId: string;
  filePath: string;
  fileName: string;
  fileSize: number;
  durationSeconds: number;
  detectedAt: string;
  whitelisted: boolean;
  uploaded: boolean;
  uploadedAt?: string;
  videoId?: string;
  videoUrl?: string;
  customTitle?: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreateRecordingPayload {
  file_path: string;
  file_name: string;
  file_size?: number;
  duration_seconds?: number;
  custom_title?: string;
}
