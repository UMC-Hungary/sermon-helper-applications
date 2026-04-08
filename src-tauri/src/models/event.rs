use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// Raw DB row for the `events` table — no platform fields, no connections, no bible refs.
/// Used only with sqlx::FromRow inside [`fetch_event`]; never serialized directly.
#[derive(Debug, FromRow)]
struct EventRow {
    pub id: Uuid,
    pub title: String,
    pub date_time: DateTime<Utc>,
    pub speaker: String,
    pub description: String,
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

/// One row from `event_bible_references`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct BibleReference {
    pub r#type: String,
    pub reference: String,
    pub translation: String,
    pub verses: serde_json::Value,
}

/// Full event including its platform connections and bible references.
/// Serialized as camelCase for API responses; deserialized from snake_case NOTIFY payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub date_time: DateTime<Utc>,
    pub speaker: String,
    pub description: String,
    pub auto_upload_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub connections: Vec<EventConnection>,
    pub bible_references: Vec<BibleReference>,
}

impl Event {
    fn from_parts(
        row: EventRow,
        connections: Vec<EventConnection>,
        bible_references: Vec<BibleReference>,
    ) -> Self {
        Self {
            id: row.id,
            title: row.title,
            date_time: row.date_time,
            speaker: row.speaker,
            description: row.description,
            auto_upload_enabled: row.auto_upload_enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
            connections,
            bible_references,
        }
    }

    /// Find a connection by platform name.
    pub fn connection(&self, platform: &str) -> Option<&EventConnection> {
        self.connections.iter().find(|c| c.platform == platform)
    }
}

/// Fetch a single event with its connections and bible references. Returns `None` if not found.
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
    let bible_references = sqlx::query_as::<_, BibleReference>(
        "SELECT type, reference, translation, verses \
         FROM event_bible_references WHERE event_id = $1 ORDER BY type",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;
    Ok(Some(Event::from_parts(row, connections, bible_references)))
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct EventSummary {
    pub id: Uuid,
    pub title: String,
    pub date_time: DateTime<Utc>,
    pub speaker: String,
    pub recording_count: i64,
    pub is_completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Find the earliest event today (UTC) that has no "completed" activity.
/// Used for auto-assigning OBS recordings.
pub async fn find_current_event(pool: &PgPool) -> anyhow::Result<Option<EventSummary>> {
    let event = sqlx::query_as::<_, EventSummary>(
        r#"
        SELECT e.id, e.title, e.date_time, e.speaker, e.created_at, e.updated_at,
               COUNT(r.id) AS recording_count,
               false AS is_completed
        FROM events e
        LEFT JOIN recordings r ON r.event_id = e.id
        WHERE DATE(e.date_time AT TIME ZONE 'UTC') = DATE(NOW() AT TIME ZONE 'UTC')
          AND NOT EXISTS (
              SELECT 1 FROM event_activities ea
              WHERE ea.event_id = e.id AND ea.activity_type = 'completed'
          )
        GROUP BY e.id
        ORDER BY e.date_time ASC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;
    Ok(event)
}

/// Connection spec in a create/update request body.
#[derive(Debug, Deserialize)]
pub struct CreateConnection {
    pub platform: String,
    pub privacy_status: Option<String>,
}

/// Bible reference spec in a create/update request body.
#[derive(Debug, Deserialize)]
pub struct CreateBibleReference {
    pub r#type: String,
    pub reference: Option<String>,
    pub translation: Option<String>,
    pub verses: Option<serde_json::Value>,
}

/// Received from frontend — stays snake_case to match JSON body.
#[derive(Debug, Deserialize)]
pub struct CreateEvent {
    pub title: String,
    pub date_time: DateTime<Utc>,
    pub speaker: Option<String>,
    pub description: Option<String>,
    pub auto_upload_enabled: Option<bool>,
    pub connections: Option<Vec<CreateConnection>>,
    pub bible_references: Option<Vec<CreateBibleReference>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEvent {
    pub title: String,
    pub date_time: DateTime<Utc>,
    pub speaker: Option<String>,
    pub description: Option<String>,
    pub auto_upload_enabled: Option<bool>,
    pub connections: Option<Vec<CreateConnection>>,
    pub bible_references: Option<Vec<CreateBibleReference>>,
}
