use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::models::{
    event::{CreateEvent, Event, EventSummary},
    recording::{CreateRecording, Recording},
};
use crate::server::AppState;

pub async fn list_events(State(state): State<AppState>) -> impl IntoResponse {
    let result = sqlx::query_as::<_, EventSummary>(
        r#"
        SELECT e.id, e.title, e.date_time, e.speaker, e.created_at, e.updated_at,
               COUNT(r.id) AS recording_count
        FROM events e
        LEFT JOIN recordings r ON r.event_id = e.id
        GROUP BY e.id
        ORDER BY e.date_time DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(events) => (StatusCode::OK, Json(events)).into_response(),
        Err(e) => {
            tracing::error!("list_events: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let event = sqlx::query_as::<_, Event>(
        "SELECT * FROM events WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await;

    match event {
        Ok(Some(e)) => (StatusCode::OK, Json(e)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("get_event: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn create_event(
    State(state): State<AppState>,
    Json(body): Json<CreateEvent>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Event>(
        r#"
        INSERT INTO events (
            title, date_time, speaker, description, textus, leckio,
            textus_translation, leckio_translation, youtube_privacy_status, auto_upload_enabled
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
    )
    .bind(&body.title)
    .bind(body.date_time)
    .bind(body.speaker.unwrap_or_default())
    .bind(body.description.unwrap_or_default())
    .bind(body.textus.unwrap_or_default())
    .bind(body.leckio.unwrap_or_default())
    .bind(body.textus_translation.unwrap_or_else(|| "RUF".to_string()))
    .bind(body.leckio_translation.unwrap_or_else(|| "RUF".to_string()))
    .bind(body.youtube_privacy_status.unwrap_or_else(|| "private".to_string()))
    .bind(body.auto_upload_enabled.unwrap_or(false))
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(event) => (StatusCode::CREATED, Json(event)).into_response(),
        Err(e) => {
            tracing::error!("create_event: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn list_recordings(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Recording>(
        "SELECT * FROM recordings WHERE event_id = $1 ORDER BY detected_at DESC",
    )
    .bind(event_id)
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(recordings) => (StatusCode::OK, Json(recordings)).into_response(),
        Err(e) => {
            tracing::error!("list_recordings: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn create_recording(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
    Json(body): Json<CreateRecording>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Recording>(
        r#"
        INSERT INTO recordings (
            event_id, file_path, file_name, file_size, duration_seconds, custom_title, detected_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(event_id)
    .bind(&body.file_path)
    .bind(&body.file_name)
    .bind(body.file_size.unwrap_or(0))
    .bind(body.duration_seconds.unwrap_or(0.0))
    .bind(body.custom_title.as_deref())
    .bind(Utc::now())
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(recording) => (StatusCode::CREATED, Json(recording)).into_response(),
        Err(e) => {
            tracing::error!("create_recording: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
