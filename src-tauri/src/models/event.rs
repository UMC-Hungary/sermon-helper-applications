use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// Raw DB row for the `events` table — no platform fields, no connections.
/// Used only with sqlx::FromRow inside [`fetch_event`]; never serialized directly.
#[derive(Debug, FromRow)]
struct EventRow {
    pub id: Uuid,
    pub title: String,
    pub date_time: DateTime<Utc>,
    pub speaker: String,
    pub description: String,
    pub textus: String,
    pub leckio: String,
    pub textus_translation: String,
    pub leckio_translation: String,
    pub auto_upload_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One row from `event_connections`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct EventConnection {
    pub platform: String,
    pub external_id: Option<String>,
    pub stream_url: Option<String>,
    pub event_url: Option<String>,
    pub schedule_status: String,
    pub privacy_status: Option<String>,
    pub extra: Option<serde_json::Value>,
}

/// Full event including its platform connections.
/// Serialized as camelCase for API responses; deserialized from snake_case NOTIFY payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
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
    pub auto_upload_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub connections: Vec<EventConnection>,
}

impl Event {
    fn from_parts(row: EventRow, connections: Vec<EventConnection>) -> Self {
        Self {
            id: row.id,
            title: row.title,
            date_time: row.date_time,
            speaker: row.speaker,
            description: row.description,
            textus: row.textus,
            leckio: row.leckio,
            textus_translation: row.textus_translation,
            leckio_translation: row.leckio_translation,
            auto_upload_enabled: row.auto_upload_enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
            connections,
        }
    }

    /// Find a connection by platform name.
    pub fn connection(&self, platform: &str) -> Option<&EventConnection> {
        self.connections.iter().find(|c| c.platform == platform)
    }
}

/// Fetch a single event with its connections. Returns `None` if not found.
pub async fn fetch_event(id: Uuid, pool: &PgPool) -> anyhow::Result<Option<Event>> {
    let row = sqlx::query_as::<_, EventRow>("SELECT * FROM events WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let connections = sqlx::query_as::<_, EventConnection>(
        "SELECT platform, external_id, stream_url, event_url, \
         schedule_status, privacy_status, extra \
         FROM event_connections WHERE event_id = $1 ORDER BY platform",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;
    Ok(Some(Event::from_parts(row, connections)))
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct EventSummary {
    pub id: Uuid,
    pub title: String,
    pub date_time: DateTime<Utc>,
    pub speaker: String,
    pub recording_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Connection spec in a create/update request body.
#[derive(Debug, Deserialize)]
pub struct CreateConnection {
    pub platform: String,
    pub privacy_status: Option<String>,
}

/// Received from frontend — stays snake_case to match JSON body.
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
    pub auto_upload_enabled: Option<bool>,
    pub connections: Option<Vec<CreateConnection>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEvent {
    pub title: String,
    pub date_time: DateTime<Utc>,
    pub speaker: Option<String>,
    pub description: Option<String>,
    pub textus: Option<String>,
    pub leckio: Option<String>,
    pub textus_translation: Option<String>,
    pub leckio_translation: Option<String>,
    pub auto_upload_enabled: Option<bool>,
    pub connections: Option<Vec<CreateConnection>>,
}
