use chrono::{DateTime, Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// How long after its start an event still counts as the one happening now.
const CURRENT_EVENT_WINDOW_MINUTES: i64 = 4 * 60;

/// Raw DB row for the `events` table — no platform fields, no connections, no bible refs.
/// Used only with sqlx::FromRow inside [`fetch_event`]; never serialized directly.
#[derive(Debug, FromRow)]
struct EventRow {
    pub id: Uuid,
    pub title: String,
    pub computed_title: String,
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
    #[serde(default)]
    pub computed_title: String,
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
            computed_title: row.computed_title,
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

    /// What gets published to the platforms. Events created before the composed
    /// title existed — and any created outside the editor — fall back to `title`.
    pub fn published_title(&self) -> &str {
        if self.computed_title.is_empty() {
            &self.title
        } else {
            &self.computed_title
        }
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
    pub computed_title: String,
    pub date_time: DateTime<Utc>,
    pub speaker: String,
    pub recording_count: i64,
    pub is_completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn fetch_event_summaries(pool: &PgPool) -> Result<Vec<EventSummary>, sqlx::Error> {
    sqlx::query_as::<_, EventSummary>(
        r#"SELECT e.id, e.title, e.computed_title, e.date_time, e.speaker,
                  e.created_at, e.updated_at,
                  COUNT(r.id) AS recording_count,
                  EXISTS (
                      SELECT 1 FROM event_activities ea
                      WHERE ea.event_id = e.id AND ea.activity_type = 'completed'
                  ) AS is_completed
           FROM events e
           LEFT JOIN recordings r ON r.event_id = e.id
           GROUP BY e.id
           ORDER BY e.date_time DESC"#,
    )
    .fetch_all(pool)
    .await
}

/// The event happening now, and the only place that decides it: the latest
/// uncompleted event that started within [`CURRENT_EVENT_WINDOW_MINUTES`] on the
/// local current date, else the next uncompleted event. A caller that needs an
/// answer when this returns `None` adds that fallback itself.
pub fn current_event(events: &[EventSummary], now: DateTime<Utc>) -> Option<&EventSummary> {
    let today = now.with_timezone(&Local).date_naive();
    let window = Duration::minutes(CURRENT_EVENT_WINDOW_MINUTES);
    events
        .iter()
        .filter(|event| {
            !event.is_completed
                && event.date_time.with_timezone(&Local).date_naive() == today
                && event.date_time <= now
                && now - event.date_time <= window
        })
        .max_by_key(|event| event.date_time)
        .or_else(|| {
            events
                .iter()
                .filter(|event| !event.is_completed && event.date_time >= now)
                .min_by_key(|event| event.date_time)
        })
}

/// The current event when it falls on today's local date — what an automatically
/// detected recording is attached to. `None` leaves that recording untracked
/// rather than filing it under an event on another day.
pub async fn find_current_event(pool: &PgPool) -> anyhow::Result<Option<EventSummary>> {
    let now = Utc::now();
    let today = now.with_timezone(&Local).date_naive();
    let events = fetch_event_summaries(pool).await?;
    Ok(current_event(&events, now)
        .filter(|event| event.date_time.with_timezone(&Local).date_naive() == today)
        .cloned())
}

/// Template for the published title. Kept in step with `DEFAULT_TITLE_TEMPLATE`
/// in packages/core-client/src/utils/title-template.ts, which does the rendering.
pub const DEFAULT_TITLE_TEMPLATE: &str =
    "{date|YYYY.MM.DD.} {title}[ | Textus: {textus}][ Lekció: {leckio}][ | {speaker}]";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleTemplate {
    pub template: String,
}

impl Default for TitleTemplate {
    fn default() -> Self {
        Self {
            template: DEFAULT_TITLE_TEMPLATE.to_string(),
        }
    }
}

/// Where generated Bible slide decks are written. Empty means "not configured";
/// the core holds the path because the core is what writes the file.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SlideFolder {
    #[serde(default)]
    pub path: String,
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
    pub computed_title: Option<String>,
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
    pub computed_title: Option<String>,
    pub date_time: DateTime<Utc>,
    pub speaker: Option<String>,
    pub description: Option<String>,
    pub auto_upload_enabled: Option<bool>,
    pub connections: Option<Vec<CreateConnection>>,
    pub bible_references: Option<Vec<CreateBibleReference>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn local_time(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        Local
            .with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single()
            .unwrap()
            .with_timezone(&Utc)
    }

    fn event(id: Uuid, date_time: DateTime<Utc>, is_completed: bool) -> EventSummary {
        EventSummary {
            id,
            title: id.to_string(),
            computed_title: String::new(),
            date_time,
            speaker: String::new(),
            recording_count: 0,
            is_completed,
            created_at: date_time,
            updated_at: date_time,
        }
    }

    fn selected(events: &[EventSummary], now: DateTime<Utc>) -> Option<Uuid> {
        current_event(events, now).map(|event| event.id)
    }

    #[test]
    fn selects_recent_same_day_past_event_before_later_future_event() {
        let now = local_time(2026, 1, 11, 11, 0);
        let service_id = Uuid::new_v4();
        let later_id = Uuid::new_v4();
        let events = vec![
            event(service_id, local_time(2026, 1, 11, 10, 0), false),
            event(later_id, local_time(2026, 1, 11, 18, 0), false),
        ];

        assert_eq!(selected(&events, now), Some(service_id));
    }

    #[test]
    fn ignores_recent_past_event_after_current_window() {
        let now = local_time(2026, 1, 11, 15, 1);
        let old_service_id = Uuid::new_v4();
        let later_id = Uuid::new_v4();
        let events = vec![
            event(old_service_id, local_time(2026, 1, 11, 10, 0), false),
            event(later_id, local_time(2026, 1, 11, 18, 0), false),
        ];

        assert_eq!(selected(&events, now), Some(later_id));
    }

    #[test]
    fn ignores_completed_recent_past_event() {
        let now = local_time(2026, 1, 11, 11, 0);
        let completed_id = Uuid::new_v4();
        let later_id = Uuid::new_v4();
        let events = vec![
            event(completed_id, local_time(2026, 1, 11, 10, 0), true),
            event(later_id, local_time(2026, 1, 11, 18, 0), false),
        ];

        assert_eq!(selected(&events, now), Some(later_id));
    }

    /// The morning service is rarely marked completed by hand, so the evening
    /// recording used to be filed under it.
    #[test]
    fn second_service_of_the_day_wins_once_it_has_started() {
        let now = local_time(2026, 1, 11, 18, 30);
        let morning_id = Uuid::new_v4();
        let evening_id = Uuid::new_v4();
        let events = vec![
            event(morning_id, local_time(2026, 1, 11, 10, 0), false),
            event(evening_id, local_time(2026, 1, 11, 18, 0), false),
        ];

        assert_eq!(selected(&events, now), Some(evening_id));
    }

    #[test]
    fn uses_the_local_date_not_the_utc_date() {
        let now = local_time(2026, 1, 11, 0, 45);
        let after_midnight_id = Uuid::new_v4();
        let events = vec![event(
            after_midnight_id,
            local_time(2026, 1, 11, 0, 30),
            false,
        )];

        assert_eq!(selected(&events, now), Some(after_midnight_id));
    }

    #[test]
    fn returns_nothing_when_every_event_is_past() {
        let now = local_time(2026, 1, 13, 9, 0);
        let events = vec![event(Uuid::new_v4(), local_time(2026, 1, 11, 10, 0), false)];

        assert_eq!(selected(&events, now), None);
    }
}
