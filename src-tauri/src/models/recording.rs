use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct Recording {
    pub id: Uuid,
    pub event_id: Uuid,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub duration_seconds: f64,
    pub detected_at: DateTime<Utc>,
    pub whitelisted: bool,
    pub uploaded: bool,
    pub uploaded_at: Option<DateTime<Utc>>,
    pub video_id: Option<String>,
    pub video_url: Option<String>,
    pub custom_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Received from frontend — stays snake_case to match JSON body
#[derive(Debug, Deserialize)]
pub struct CreateRecording {
    pub file_path: String,
    pub file_name: String,
    pub file_size: Option<i64>,
    pub duration_seconds: Option<f64>,
    pub custom_title: Option<String>,
}
