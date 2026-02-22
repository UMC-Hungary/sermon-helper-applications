use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub date_time: DateTime<Utc>,
    pub speaker: String,
    pub description: String,
    pub textus: String,
    pub leckio: String,
    pub textus_translation: String,
    pub leckio_translation: String,
    pub youtube_privacy_status: String,
    pub auto_upload_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventSummary {
    pub id: Uuid,
    pub title: String,
    pub date_time: DateTime<Utc>,
    pub speaker: String,
    pub recording_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEvent {
    pub title: String,
    pub date_time: DateTime<Utc>,
    pub speaker: Option<String>,
    pub description: Option<String>,
    pub textus: Option<String>,
    pub leckio: Option<String>,
    pub textus_translation: Option<String>,
    pub leckio_translation: Option<String>,
    pub youtube_privacy_status: Option<String>,
    pub auto_upload_enabled: Option<bool>,
}
