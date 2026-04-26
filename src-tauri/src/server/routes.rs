use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::Ordering;
use uuid::Uuid;

use crate::connectors::{facebook, youtube};
use crate::models::{
    activity::{self, CreateEventActivity},
    cron_job::{self, CreateCronJob, UpdateCronJob},
    event::{fetch_event, CreateBibleReference, CreateEvent, EventSummary, UpdateEvent},
    recording::{CreateRecording, FlagUploadRequest, Recording, RecordingUpload},
    untracked_recording,
};
use crate::server::websocket::{
    broadcast_event_changed, broadcast_untracked_removed, spawn_scheduling_tasks,
};
use crate::server::AppState;
use crate::server::OAUTH_REDIRECT_URI;

const OAUTH_SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Authentication Successful</title>
  <style>
    body { font-family: system-ui, sans-serif; display: flex; align-items: center;
           justify-content: center; height: 100vh; margin: 0; background: #f9fafb; }
    .card { text-align: center; padding: 2rem 3rem; background: #fff;
            border-radius: 0.75rem; box-shadow: 0 2px 8px rgba(0,0,0,.08); }
    h1 { color: #065f46; margin-bottom: 0.5rem; font-size: 1.5rem; }
    p { color: #6b7280; margin: 0; }
  </style>
</head>
<body>
  <div class="card">
    <h1>Authentication Successful</h1>
    <p>You can close this tab and return to the app.</p>
  </div>
  <script>setTimeout(() => window.close(), 3000);</script>
</body>
</html>"#;

const OAUTH_ERROR_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Authentication Failed</title>
  <style>
    body { font-family: system-ui, sans-serif; display: flex; align-items: center;
           justify-content: center; height: 100vh; margin: 0; background: #f9fafb; }
    .card { text-align: center; padding: 2rem 3rem; background: #fff;
            border-radius: 0.75rem; box-shadow: 0 2px 8px rgba(0,0,0,.08); }
    h1 { color: #991b1b; margin-bottom: 0.5rem; font-size: 1.5rem; }
    p { color: #6b7280; margin: 0; }
  </style>
</head>
<body>
  <div class="card">
    <h1>Authentication Failed</h1>
    <p>An error occurred. Please close this tab and try again.</p>
  </div>
</body>
</html>"#;

// ── Connector statuses ────────────────────────────────────────────────────────

pub async fn get_connector_state(State(state): State<AppState>) -> impl IntoResponse {
    let obs_state = state.obs_connector.get_output_state().await;
    Json(json!({
        "obs": obs_state.map(|s| json!({"isStreaming": s.is_streaming, "isRecording": s.is_recording}))
    }))
}

pub async fn get_connector_statuses(State(state): State<AppState>) -> impl IntoResponse {
    let obs = state.obs_connector.get_status().await;
    let vmix = state.vmix_connector.get_status();
    let yt = state.youtube_connector.get_status().await;
    let fb = state.facebook_connector.get_status().await;
    Json(json!({ "obs": obs, "vmix": vmix, "youtube": yt, "facebook": fb }))
}

// ── YouTube OAuth ─────────────────────────────────────────────────────────────

pub async fn youtube_auth_url(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.youtube_config.read().await.clone();
    if config.client_id.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "YouTube not configured"}))).into_response();
    }

    let state_token = Uuid::new_v4().to_string();
    {
        let mut states = state.oauth_states.write().await;
        states.insert(state_token.clone(), ("youtube".to_string(), std::time::Instant::now()));
    }

    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=https://www.googleapis.com/auth/youtube&access_type=offline&prompt=consent&state={}",
        urlencoding::encode(&config.client_id),
        urlencoding::encode(OAUTH_REDIRECT_URI),
        urlencoding::encode(&state_token),
    );
    Json(json!({ "url": url })).into_response()
}

#[derive(Deserialize)]
pub struct OAuthCallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

/// Unified OAuth callback — handles both YouTube and Facebook.
/// Google/Facebook redirect to http://127.0.0.1:8766/callback; the connector
/// is identified by looking up the state token in oauth_states.
pub async fn oauth_callback(
    State(state): State<AppState>,
    Query(params): Query<OAuthCallbackParams>,
) -> impl IntoResponse {
    if params.error.is_some() {
        return Html(OAUTH_ERROR_HTML).into_response();
    }
    let (code, state_token) = match (params.code, params.state) {
        (Some(c), Some(s)) => (c, s),
        _ => return Html(OAUTH_ERROR_HTML).into_response(),
    };

    let connector = {
        let mut states = state.oauth_states.write().await;
        match states.remove(&state_token) {
            Some((name, _)) => name,
            None => return Html(OAUTH_ERROR_HTML).into_response(),
        }
    };

    match connector.as_str() {
        "youtube" => {
            let config = state.youtube_config.read().await.clone();
            match youtube::exchange_code(&state.pool, &config, &code, OAUTH_REDIRECT_URI).await {
                Ok(_) => {
                    if let Some(handle) = state.app_handle.clone() {
                        state.youtube_connector.start(state.pool.clone(), config, handle).await;
                    }
                    Html(OAUTH_SUCCESS_HTML).into_response()
                }
                Err(e) => {
                    tracing::error!("YouTube token exchange failed: {e}");
                    Html(OAUTH_ERROR_HTML).into_response()
                }
            }
        }
        "facebook" => {
            let config = state.facebook_config.read().await.clone();
            match facebook::exchange_code(&state.pool, &config, &code, OAUTH_REDIRECT_URI).await {
                Ok(_) => {
                    if let Some(handle) = state.app_handle.clone() {
                        state.facebook_connector.start(state.pool.clone(), handle).await;
                    }
                    Html(OAUTH_SUCCESS_HTML).into_response()
                }
                Err(e) => {
                    tracing::error!("Facebook token exchange failed: {e}");
                    Html(OAUTH_ERROR_HTML).into_response()
                }
            }
        }
        _ => Html(OAUTH_ERROR_HTML).into_response(),
    }
}

pub async fn youtube_logout(State(state): State<AppState>) -> impl IntoResponse {
    if let Err(e) = youtube::delete_tokens(&state.pool).await {
        tracing::error!("YouTube logout: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    state.youtube_connector.stop().await;
    StatusCode::NO_CONTENT.into_response()
}

// ── Facebook OAuth ────────────────────────────────────────────────────────────

pub async fn facebook_auth_url(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.facebook_config.read().await.clone();
    if config.app_id.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Facebook not configured"}))).into_response();
    }

    let state_token = Uuid::new_v4().to_string();
    {
        let mut states = state.oauth_states.write().await;
        states.insert(state_token.clone(), ("facebook".to_string(), std::time::Instant::now()));
    }

    let url = format!(
        "https://www.facebook.com/v19.0/dialog/oauth?client_id={}&redirect_uri={}&scope=pages_manage_posts,pages_read_engagement,publish_video&state={}",
        urlencoding::encode(&config.app_id),
        urlencoding::encode(OAUTH_REDIRECT_URI),
        urlencoding::encode(&state_token),
    );
    Json(json!({ "url": url })).into_response()
}

pub async fn facebook_logout(State(state): State<AppState>) -> impl IntoResponse {
    if let Err(e) = facebook::delete_tokens(&state.pool).await {
        tracing::error!("Facebook logout: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    state.facebook_connector.stop().await;
    StatusCode::NO_CONTENT.into_response()
}

// ── Manual schedule triggers ──────────────────────────────────────────────────

pub async fn trigger_youtube_schedule(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> impl IntoResponse {
    let event = match fetch_event(event_id, &state.pool).await {
        Ok(Some(e)) => e,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("trigger_youtube_schedule fetch: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let token = match youtube::load_tokens(&state.pool).await {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Not authenticated"}))).into_response(),
    };

    let yt_conn = event.connection("youtube");
    let existing_id = yt_conn.and_then(|c| c.external_id.as_deref());
    let privacy_status = yt_conn
        .and_then(|c| c.privacy_status.as_deref())
        .unwrap_or("private");

    match youtube::schedule_event(
        &event.id.to_string(),
        &event.title,
        &event.date_time,
        &token.access_token,
        existing_id,
        privacy_status,
    )
    .await
    {
        Ok(result) => {
            let write_result = async {
                let mut tx = state.pool.begin().await?;
                sqlx::query("SET LOCAL app.skip_sync_notify = 'true'")
                    .execute(&mut *tx)
                    .await?;
                sqlx::query(
                    r#"INSERT INTO event_connections (event_id, platform, external_id, stream_url, schedule_status)
                       VALUES ($1, 'youtube', $2, $3, 'scheduled')
                       ON CONFLICT (event_id, platform) DO UPDATE SET
                           external_id     = EXCLUDED.external_id,
                           stream_url      = EXCLUDED.stream_url,
                           schedule_status = 'scheduled',
                           updated_at      = NOW()"#,
                )
                .bind(event_id)
                .bind(&result.broadcast_id)
                .bind(&result.watch_url)
                .execute(&mut *tx)
                .await?;
                sqlx::query("UPDATE events SET updated_at = NOW() WHERE id = $1")
                    .bind(event_id)
                    .execute(&mut *tx)
                    .await?;
                tx.commit().await?;
                anyhow::Ok(())
            }
            .await;
            if let Err(e) = write_result {
                tracing::error!("Failed to persist YouTube broadcast result: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            Json(result).into_response()
        }
        Err(e) => {
            tracing::error!("YouTube schedule_event failed: {e}");
            let _ = async {
                let mut tx = state.pool.begin().await?;
                sqlx::query("SET LOCAL app.skip_sync_notify = 'true'")
                    .execute(&mut *tx)
                    .await?;
                sqlx::query(
                    r#"INSERT INTO event_connections (event_id, platform, schedule_status)
                       VALUES ($1, 'youtube', 'failed')
                       ON CONFLICT (event_id, platform) DO UPDATE SET
                           schedule_status = 'failed',
                           updated_at      = NOW()"#,
                )
                .bind(event_id)
                .execute(&mut *tx)
                .await?;
                sqlx::query("UPDATE events SET updated_at = NOW() WHERE id = $1")
                    .bind(event_id)
                    .execute(&mut *tx)
                    .await?;
                tx.commit().await?;
                anyhow::Ok(())
            }
            .await;
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
        }
    }
}

// ── YouTube channel content ───────────────────────────────────────────────────

pub async fn get_youtube_content(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.youtube_config.read().await.clone();
    match youtube::fetch_channel_content(&state.pool, &config).await {
        Ok(content) => Json(content).into_response(),
        Err(e) => {
            tracing::error!("fetch_channel_content failed: {e}");
            if e.is::<youtube::AuthRequired>() {
                // Tokens were already deleted by fetch_channel_content; stop
                // the connector loop so the frontend sees the status change.
                state.youtube_connector.stop().await;
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "auth_required", "message": "Re-login required" })),
                )
                    .into_response();
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

pub async fn trigger_facebook_schedule(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> impl IntoResponse {
    let event = match fetch_event(event_id, &state.pool).await {
        Ok(Some(e)) => e,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("trigger_facebook_schedule fetch: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let token = match facebook::load_tokens(&state.pool).await {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Not authenticated"}))).into_response(),
    };

    let config = state.facebook_config.read().await.clone();
    if config.page_id.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Facebook page_id not configured"}))).into_response();
    }

    let fb_conn = event.connection("facebook");
    let privacy_status = fb_conn
        .and_then(|c| c.privacy_status.as_deref())
        .unwrap_or("EVERYONE");

    match facebook::schedule_event(
        &event.title,
        &event.date_time,
        &token.access_token,
        &config.page_id,
        privacy_status,
    )
    .await
    {
        Ok(result) => {
            let extra = serde_json::json!({"stream_id": result.stream_id});
            let write_result = async {
                let mut tx = state.pool.begin().await?;
                sqlx::query("SET LOCAL app.skip_sync_notify = 'true'")
                    .execute(&mut *tx)
                    .await?;
                sqlx::query(
                    r#"INSERT INTO event_connections (event_id, platform, external_id, event_url, schedule_status, extra)
                       VALUES ($1, 'facebook', $2, $3, 'scheduled', $4)
                       ON CONFLICT (event_id, platform) DO UPDATE SET
                           external_id     = EXCLUDED.external_id,
                           event_url       = EXCLUDED.event_url,
                           schedule_status = 'scheduled',
                           extra           = EXCLUDED.extra,
                           updated_at      = NOW()"#,
                )
                .bind(event_id)
                .bind(&result.event_id)
                .bind(&result.event_url)
                .bind(extra)
                .execute(&mut *tx)
                .await?;
                sqlx::query("UPDATE events SET updated_at = NOW() WHERE id = $1")
                    .bind(event_id)
                    .execute(&mut *tx)
                    .await?;
                tx.commit().await?;
                anyhow::Ok(())
            }
            .await;
            if let Err(e) = write_result {
                tracing::error!("Failed to persist Facebook schedule result: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            Json(result).into_response()
        }
        Err(e) => {
            tracing::error!("Facebook schedule_event failed: {e}");
            let _ = async {
                let mut tx = state.pool.begin().await?;
                sqlx::query("SET LOCAL app.skip_sync_notify = 'true'")
                    .execute(&mut *tx)
                    .await?;
                sqlx::query(
                    r#"INSERT INTO event_connections (event_id, platform, schedule_status)
                       VALUES ($1, 'facebook', 'failed')
                       ON CONFLICT (event_id, platform) DO UPDATE SET
                           schedule_status = 'failed',
                           updated_at      = NOW()"#,
                )
                .bind(event_id)
                .execute(&mut *tx)
                .await?;
                sqlx::query("UPDATE events SET updated_at = NOW() WHERE id = $1")
                    .bind(event_id)
                    .execute(&mut *tx)
                    .await?;
                tx.commit().await?;
                anyhow::Ok(())
            }
            .await;
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
        }
    }
}



// ── Multi-stream relay: stream key fetch ──────────────────────────────────────

/// Fetch the default ingestion (stream) key for the authenticated YouTube channel.
/// Returns `{ rtmpUrl: "rtmp://a.rtmp.youtube.com/live2/STREAM_KEY" }`.
pub async fn get_youtube_stream_key(State(state): State<AppState>) -> impl IntoResponse {
    let token = match youtube::load_tokens(&state.pool).await {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Not authenticated with YouTube"})),
            )
                .into_response()
        }
    };

    #[derive(Deserialize)]
    struct IngestionInfo {
        #[serde(rename = "ingestionAddress")]
        ingestion_address: String,
        #[serde(rename = "streamName")]
        stream_name: String,
    }

    #[derive(Deserialize)]
    struct Cdn {
        #[serde(rename = "ingestionInfo")]
        ingestion_info: IngestionInfo,
    }

    #[derive(Deserialize)]
    struct StreamItem {
        cdn: Cdn,
    }

    #[derive(Deserialize)]
    struct StreamList {
        items: Option<Vec<StreamItem>>,
    }

    let client = reqwest::Client::new();
    let resp = client
        .get("https://www.googleapis.com/youtube/v3/liveStreams")
        .query(&[("part", "cdn"), ("mine", "true")])
        .bearer_auth(&token.access_token)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => match r.json::<StreamList>().await {
            Ok(list) => match list.items.and_then(|items| items.into_iter().next()) {
                Some(item) => {
                    let rtmp_url = format!(
                        "{}/{}",
                        item.cdn.ingestion_info.ingestion_address,
                        item.cdn.ingestion_info.stream_name
                    );
                    Json(json!({ "rtmpUrl": rtmp_url })).into_response()
                }
                None => (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "No YouTube live stream found for this account. Make sure you have a live stream set up in YouTube Studio."})),
                )
                    .into_response(),
            },
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response(),
        },
        Ok(r) => {
            let status = r.status();
            let detail = r.text().await.unwrap_or_default();
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": format!("YouTube API {status}: {detail}")})),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Fetch the RTMP stream URL for the first scheduled Facebook live video on the page.
/// Returns `{ rtmpUrl: "rtmps://live-api-s.facebook.com:443/rtmp/STREAM_KEY" }`.
pub async fn get_facebook_stream_key(State(state): State<AppState>) -> impl IntoResponse {
    let token = match facebook::load_tokens(&state.pool).await {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Not authenticated with Facebook"})),
            )
                .into_response()
        }
    };

    let config = state.facebook_config.read().await.clone();
    if config.page_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Facebook page_id not configured"})),
        )
            .into_response();
    }

    #[derive(Deserialize)]
    struct LiveVideo {
        secure_stream_url: Option<String>,
    }

    #[derive(Deserialize)]
    struct LiveVideoList {
        data: Vec<LiveVideo>,
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "https://graph.facebook.com/v19.0/{}/live_videos",
            config.page_id
        ))
        .query(&[
            ("fields", "secure_stream_url"),
            ("status", "SCHEDULED_UNPUBLISHED"),
            ("access_token", token.access_token.as_str()),
        ])
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => match r.json::<LiveVideoList>().await {
            Ok(list) => match list.data.into_iter().next().and_then(|v| v.secure_stream_url) {
                Some(url) => Json(json!({ "rtmpUrl": url })).into_response(),
                None => (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "No scheduled Facebook live video found. Create a live event in the app first."})),
                )
                    .into_response(),
            },
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response(),
        },
        Ok(r) => {
            let status = r.status();
            let detail = r.text().await.unwrap_or_default();
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": format!("Facebook API {status}: {detail}")})),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn list_events(State(state): State<AppState>) -> impl IntoResponse {
    let result = sqlx::query_as::<_, EventSummary>(
        r#"
        SELECT e.id, e.title, e.date_time, e.speaker, e.created_at, e.updated_at,
               COUNT(r.id) AS recording_count,
               EXISTS (
                   SELECT 1 FROM event_activities ea
                   WHERE ea.event_id = e.id AND ea.activity_type = 'completed'
               ) AS is_completed
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
    match fetch_event(id, &state.pool).await {
        Ok(Some(event)) => (StatusCode::OK, Json(event)).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("get_event: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Upsert (or delete) bible references for an event inside an open transaction.
/// An entry with an empty reference string is deleted; non-empty entries are upserted.
async fn upsert_bible_references(
    event_id: Uuid,
    refs: &Option<Vec<CreateBibleReference>>,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> anyhow::Result<()> {
    let Some(refs) = refs else { return Ok(()) };
    for br in refs {
        let reference = br.reference.as_deref().unwrap_or("").trim();
        if reference.is_empty() {
            sqlx::query(
                "DELETE FROM event_bible_references WHERE event_id = $1 AND type = $2",
            )
            .bind(event_id)
            .bind(&br.r#type)
            .execute(&mut **tx)
            .await?;
        } else {
            let translation = br.translation.as_deref().unwrap_or("UF");
            let verses = br
                .verses
                .clone()
                .unwrap_or_else(|| serde_json::json!([]));
            sqlx::query(
                "INSERT INTO event_bible_references (event_id, type, reference, translation, verses) \
                 VALUES ($1, $2, $3, $4, $5) \
                 ON CONFLICT (event_id, type) DO UPDATE SET \
                   reference = EXCLUDED.reference, \
                   translation = EXCLUDED.translation, \
                   verses = EXCLUDED.verses, \
                   updated_at = NOW()",
            )
            .bind(event_id)
            .bind(&br.r#type)
            .bind(reference)
            .bind(translation)
            .bind(verses)
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

pub async fn create_event(
    State(state): State<AppState>,
    Json(body): Json<CreateEvent>,
) -> impl IntoResponse {
    let result: anyhow::Result<_> = async {
        let mut tx = state.pool.begin().await?;
        sqlx::query("SET LOCAL app.skip_sync_notify = 'true'")
            .execute(&mut *tx)
            .await?;

        let event_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO events (title, date_time, speaker, description, auto_upload_enabled)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id"#,
        )
        .bind(&body.title)
        .bind(body.date_time)
        .bind(body.speaker.unwrap_or_default())
        .bind(body.description.unwrap_or_default())
        .bind(body.auto_upload_enabled.unwrap_or(false))
        .fetch_one(&mut *tx)
        .await?;

        // Default connections: youtube (private) and facebook (EVERYONE).
        // Override privacy if provided in the request body.
        let mut conn_map: Vec<(String, String)> = vec![
            ("youtube".to_string(), "private".to_string()),
            ("facebook".to_string(), "EVERYONE".to_string()),
        ];
        if let Some(req_conns) = &body.connections {
            for c in req_conns {
                if let Some(entry) = conn_map.iter_mut().find(|(p, _)| p == &c.platform) {
                    if let Some(ps) = &c.privacy_status {
                        entry.1 = ps.clone();
                    }
                } else {
                    conn_map.push((
                        c.platform.clone(),
                        c.privacy_status.clone().unwrap_or_default(),
                    ));
                }
            }
        }

        for (platform, privacy) in &conn_map {
            sqlx::query(
                "INSERT INTO event_connections (event_id, platform, privacy_status) VALUES ($1, $2, $3)",
            )
            .bind(event_id)
            .bind(platform)
            .bind(privacy)
            .execute(&mut *tx)
            .await?;
        }

        upsert_bible_references(event_id, &body.bible_references, &mut tx).await?;

        tx.commit().await?;

        let event = fetch_event(event_id, &state.pool)
            .await?
            .ok_or_else(|| anyhow::anyhow!("event {event_id} not found after create"))?;
        anyhow::Ok(event)
    }
    .await;

    match result {
        Ok(event) => {
            broadcast_event_changed(&state, "INSERT", &event).await;
            spawn_scheduling_tasks(event.clone(), state);
            (StatusCode::CREATED, Json(event)).into_response()
        }
        Err(e) => {
            tracing::error!("create_event: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn update_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateEvent>,
) -> impl IntoResponse {
    let result: anyhow::Result<Option<_>> = async {
        let mut tx = state.pool.begin().await?;
        sqlx::query("SET LOCAL app.skip_sync_notify = 'true'")
            .execute(&mut *tx)
            .await?;

        let updated_id: Option<Uuid> = sqlx::query_scalar(
            r#"UPDATE events SET
                title = $1,
                date_time = $2,
                speaker = $3,
                description = $4,
                auto_upload_enabled = $5,
                updated_at = NOW()
            WHERE id = $6
            RETURNING id"#,
        )
        .bind(&body.title)
        .bind(body.date_time)
        .bind(body.speaker.unwrap_or_default())
        .bind(body.description.unwrap_or_default())
        .bind(body.auto_upload_enabled.unwrap_or(false))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        if updated_id.is_none() {
            tx.rollback().await?;
            return Ok(None);
        }

        if let Some(conns) = &body.connections {
            for conn in conns {
                if let Some(ps) = &conn.privacy_status {
                    sqlx::query(
                        "UPDATE event_connections SET privacy_status = $1, updated_at = NOW() \
                         WHERE event_id = $2 AND platform = $3",
                    )
                    .bind(ps)
                    .bind(id)
                    .bind(&conn.platform)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }

        upsert_bible_references(id, &body.bible_references, &mut tx).await?;

        tx.commit().await?;

        let event = fetch_event(id, &state.pool).await?;
        anyhow::Ok(event)
    }
    .await;

    match result {
        Ok(Some(event)) => {
            broadcast_event_changed(&state, "UPDATE", &event).await;
            spawn_scheduling_tasks(event.clone(), state);
            (StatusCode::OK, Json(event)).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("update_event: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct AllRecordingsQuery {
    filter: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingWithEvent {
    #[serde(flatten)]
    recording: Recording,
    event_title: String,
}

pub async fn list_all_recordings(
    State(state): State<AppState>,
    Query(params): Query<AllRecordingsQuery>,
) -> impl IntoResponse {
    let where_clause = match params.filter.as_deref().unwrap_or("") {
        // Never flagged and no upload history at all
        "not_flagged" => "r.uploadable = false AND NOT EXISTS (SELECT 1 FROM recording_uploads WHERE recording_id = r.id)",
        // Flagged for upload but no active or completed upload yet
        "flagged" => "r.uploadable = true AND NOT EXISTS (SELECT 1 FROM recording_uploads WHERE recording_id = r.id AND state IN ('uploading','paused','completed','failed'))",
        // Upload started and currently active/failed
        "in_progress" => "EXISTS (SELECT 1 FROM recording_uploads WHERE recording_id = r.id AND state IN ('uploading','paused','failed'))",
        // At least one platform upload completed (r.uploaded may not be set)
        "uploaded" => "EXISTS (SELECT 1 FROM recording_uploads WHERE recording_id = r.id AND state = 'completed')",
        _ => "true",
    };
    let sql = format!(
        "SELECT r.*, e.title AS _event_title \
         FROM recordings r JOIN events e ON e.id = r.event_id \
         WHERE {where_clause} \
         ORDER BY r.detected_at DESC LIMIT 100"
    );

    #[derive(sqlx::FromRow)]
    struct RecordingRow {
        #[sqlx(flatten)]
        recording: Recording,
        _event_title: String,
    }

    let rows = match sqlx::query_as::<_, RecordingRow>(&sql)
        .fetch_all(&state.pool)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("list_all_recordings: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut results: Vec<RecordingWithEvent> = rows
        .into_iter()
        .map(|row| RecordingWithEvent {
            event_title: row._event_title.clone(),
            recording: row.recording,
        })
        .collect();

    if !results.is_empty() {
        let ids: Vec<Uuid> = results.iter().map(|r| r.recording.id).collect();
        let uploads = sqlx::query_as::<_, RecordingUpload>(
            "SELECT recording_id, platform, state, progress_bytes, total_bytes, \
             visibility, video_id, video_url, error, started_at, completed_at, updated_at \
             FROM recording_uploads WHERE recording_id = ANY($1)",
        )
        .bind(&ids)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

        for item in &mut results {
            item.recording.uploads = uploads
                .iter()
                .filter(|u| u.recording_id == item.recording.id)
                .cloned()
                .collect();
        }
    }

    (StatusCode::OK, Json(results)).into_response()
}

pub async fn list_recordings(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> impl IntoResponse {
    let mut recordings = match sqlx::query_as::<_, Recording>(
        "SELECT * FROM recordings WHERE event_id = $1 ORDER BY detected_at DESC",
    )
    .bind(event_id)
    .fetch_all(&state.pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("list_recordings: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !recordings.is_empty() {
        let ids: Vec<Uuid> = recordings.iter().map(|r| r.id).collect();
        let uploads = sqlx::query_as::<_, RecordingUpload>(
            "SELECT recording_id, platform, state, progress_bytes, total_bytes, \
             visibility, video_id, video_url, error, started_at, completed_at, updated_at \
             FROM recording_uploads WHERE recording_id = ANY($1)",
        )
        .bind(&ids)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

        for rec in &mut recordings {
            rec.uploads = uploads
                .iter()
                .filter(|u| u.recording_id == rec.id)
                .cloned()
                .collect();
        }
    }

    (StatusCode::OK, Json(recordings)).into_response()
}

pub async fn create_recording(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
    Json(body): Json<CreateRecording>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Recording>(
        r#"
        INSERT INTO recordings (
            event_id, file_path, file_name, file_size, duration_seconds,
            custom_title, custom_description, detected_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(event_id)
    .bind(&body.file_path)
    .bind(&body.file_name)
    .bind(body.file_size.unwrap_or(0))
    .bind(body.duration_seconds.unwrap_or(0.0))
    .bind(body.custom_title.as_deref())
    .bind(body.custom_description.as_deref())
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

#[derive(Deserialize)]
pub struct DeleteRecordingParams {
    #[serde(default)]
    pub delete_file: bool,
}

pub async fn delete_recording(
    State(state): State<AppState>,
    Path((event_id, recording_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<DeleteRecordingParams>,
) -> impl IntoResponse {
    let row = sqlx::query_as::<_, Recording>(
        "SELECT * FROM recordings WHERE id = $1 AND event_id = $2",
    )
    .bind(recording_id)
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some(rec)) => {
            let del = sqlx::query("DELETE FROM recordings WHERE id = $1")
                .bind(recording_id)
                .execute(&state.pool)
                .await;
            if let Err(e) = del {
                tracing::error!("delete_recording DB: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            if params.delete_file {
                if let Err(e) = tokio::fs::remove_file(&rec.file_path).await {
                    tracing::warn!(
                        "delete_recording: could not delete file {}: {e}",
                        rec.file_path
                    );
                }
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("delete_recording fetch: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn delete_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let event = match fetch_event(id, &state.pool).await {
        Ok(Some(e)) => e,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("delete_event fetch: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let result = sqlx::query("DELETE FROM events WHERE id = $1 RETURNING id")
        .bind(id)
        .fetch_optional(&state.pool)
        .await;

    match result {
        Ok(Some(_)) => {
            broadcast_event_changed(&state, "DELETE", &event).await;
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("delete_event: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ── Cron Jobs ─────────────────────────────────────────────────────────────────

pub async fn list_cron_jobs(State(state): State<AppState>) -> impl IntoResponse {
    match cron_job::list_all(&state.pool).await {
        Ok(jobs) => Json(jobs).into_response(),
        Err(e) => {
            tracing::error!("list_cron_jobs: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn create_cron_job(
    State(state): State<AppState>,
    Json(body): Json<CreateCronJob>,
) -> impl IntoResponse {
    // Validate the cron expression before inserting.
    if tokio_cron_scheduler::Job::new_async(body.cron_expression.as_str(), |_, _| {
        Box::pin(async {})
    })
    .is_err()
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid cron expression"})),
        )
            .into_response();
    }

    let result: anyhow::Result<cron_job::CronJob> = async {
        let mut tx = state.pool.begin().await?;

        let row = sqlx::query_as::<_, (Uuid, String, String, bool, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
            "INSERT INTO cron_jobs (name, cron_expression, enabled) \
             VALUES ($1, $2, $3) \
             RETURNING id, name, cron_expression, enabled, created_at, updated_at",
        )
        .bind(&body.name)
        .bind(&body.cron_expression)
        .bind(body.enabled)
        .fetch_one(&mut *tx)
        .await?;

        cron_job::sync_features(&mut tx, row.0, body.pull_youtube, body.auto_upload).await?;
        tx.commit().await?;

        Ok(cron_job::CronJob {
            id: row.0,
            name: row.1,
            cron_expression: row.2,
            enabled: row.3,
            pull_youtube: body.pull_youtube,
            auto_upload: body.auto_upload,
            created_at: row.4,
            updated_at: row.5,
        })
    }
    .await;

    match result {
        Ok(job) => {
            let pool = state.pool.clone();
            let clients = state.ws_clients.clone();
            let yt = state.youtube_connector.clone();
            let sched = state.cron_scheduler.clone();
            let us = state.upload_service.clone();
            tokio::spawn(async move {
                sched.reload(pool, clients, yt, us).await;
            });
            (StatusCode::CREATED, Json(job)).into_response()
        }
        Err(e) => {
            tracing::error!("create_cron_job: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn update_cron_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateCronJob>,
) -> impl IntoResponse {
    // Validate the cron expression before updating.
    if tokio_cron_scheduler::Job::new_async(body.cron_expression.as_str(), |_, _| {
        Box::pin(async {})
    })
    .is_err()
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid cron expression"})),
        )
            .into_response();
    }

    let result: anyhow::Result<Option<cron_job::CronJob>> = async {
        let mut tx = state.pool.begin().await?;

        let row = sqlx::query_as::<_, (Uuid, String, String, bool, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
            "UPDATE cron_jobs \
             SET name = $1, cron_expression = $2, enabled = $3, updated_at = NOW() \
             WHERE id = $4 \
             RETURNING id, name, cron_expression, enabled, created_at, updated_at",
        )
        .bind(&body.name)
        .bind(&body.cron_expression)
        .bind(body.enabled)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(row) = row else {
            tx.rollback().await?;
            return Ok(None);
        };

        cron_job::sync_features(&mut tx, row.0, body.pull_youtube, body.auto_upload).await?;
        tx.commit().await?;

        Ok(Some(cron_job::CronJob {
            id: row.0,
            name: row.1,
            cron_expression: row.2,
            enabled: row.3,
            pull_youtube: body.pull_youtube,
            auto_upload: body.auto_upload,
            created_at: row.4,
            updated_at: row.5,
        }))
    }
    .await;

    match result {
        Ok(Some(job)) => {
            let pool = state.pool.clone();
            let clients = state.ws_clients.clone();
            let yt = state.youtube_connector.clone();
            let sched = state.cron_scheduler.clone();
            let us = state.upload_service.clone();
            tokio::spawn(async move {
                sched.reload(pool, clients, yt, us).await;
            });
            Json(job).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("update_cron_job: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn delete_cron_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let result =
        sqlx::query("DELETE FROM cron_jobs WHERE id = $1 RETURNING id")
            .bind(id)
            .fetch_optional(&state.pool)
            .await;

    match result {
        Ok(Some(_)) => {
            let pool = state.pool.clone();
            let clients = state.ws_clients.clone();
            let yt = state.youtube_connector.clone();
            let sched = state.cron_scheduler.clone();
            let us = state.upload_service.clone();
            tokio::spawn(async move {
                sched.reload(pool, clients, yt, us).await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("delete_cron_job: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ── Upload flag & trigger ─────────────────────────────────────────────────────

pub async fn flag_upload(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
    Json(body): Json<FlagUploadRequest>,
) -> impl IntoResponse {
    let result: anyhow::Result<()> = async {
        for item in &body.recordings {
            // Mark the recording as uploadable, optionally update custom title/description.
            sqlx::query(
                "UPDATE recordings SET uploadable = true, \
                 custom_title = COALESCE($1, custom_title), \
                 custom_description = COALESCE($2, custom_description), \
                 updated_at = NOW() \
                 WHERE id = $3 AND event_id = $4",
            )
            .bind(item.custom_title.as_deref())
            .bind(item.custom_description.as_deref())
            .bind(item.recording_id)
            .bind(event_id)
            .execute(&state.pool)
            .await?;

            // Insert/update upload rows for each requested platform.
            for platform in &item.platforms {
                let visibility = if platform == "youtube" {
                    item.youtube_visibility
                        .as_deref()
                        .unwrap_or("private")
                        .to_string()
                } else {
                    item.facebook_visibility
                        .as_deref()
                        .unwrap_or("ONLY_ME")
                        .to_string()
                };

                sqlx::query(
                    "INSERT INTO recording_uploads (recording_id, platform, state, visibility, updated_at) \
                     VALUES ($1, $2, 'pending', $3, NOW()) \
                     ON CONFLICT (recording_id, platform) DO UPDATE SET \
                         state = CASE WHEN recording_uploads.state = 'completed' \
                                      THEN 'completed' ELSE 'pending' END, \
                         visibility = EXCLUDED.visibility, \
                         updated_at = NOW()",
                )
                .bind(item.recording_id)
                .bind(platform)
                .bind(&visibility)
                .execute(&state.pool)
                .await?;
            }
        }
        Ok(())
    }
    .await;

    match result {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("flag_upload: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn trigger_upload_cycle(State(state): State<AppState>) -> impl IntoResponse {
    let us = state.upload_service.clone();
    tokio::spawn(async move {
        us.run_cycle().await;
    });
    StatusCode::NO_CONTENT.into_response()
}

// ── Untracked recordings ───────────────────────────────────────────────────────

pub async fn list_untracked_recordings(State(state): State<AppState>) -> impl IntoResponse {
    match untracked_recording::list_untracked(&state.pool).await {
        Ok(recordings) => (StatusCode::OK, Json(recordings)).into_response(),
        Err(e) => {
            tracing::error!("list_untracked_recordings: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct AssignRecordingBody {
    pub event_id: Uuid,
}

pub async fn assign_untracked_recording(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<AssignRecordingBody>,
) -> impl IntoResponse {
    let result: anyhow::Result<Recording> = async {
        // Fetch the untracked row (404 if missing)
        let untracked = sqlx::query_as::<_, untracked_recording::UntrackedRecording>(
            "SELECT * FROM untracked_recordings WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("NOT_FOUND"))?;

        // Verify event exists
        let event_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM events WHERE id = $1)")
                .bind(body.event_id)
                .fetch_one(&state.pool)
                .await?;
        if !event_exists {
            return Err(anyhow::anyhow!("EVENT_NOT_FOUND"));
        }

        let mut tx = state.pool.begin().await?;
        let recording = sqlx::query_as::<_, Recording>(
            r#"INSERT INTO recordings (event_id, file_path, file_name, file_size, duration_seconds, detected_at)
               VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#,
        )
        .bind(body.event_id)
        .bind(&untracked.file_path)
        .bind(&untracked.file_name)
        .bind(untracked.file_size)
        .bind(untracked.duration_seconds)
        .bind(untracked.detected_at)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM untracked_recordings WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(recording)
    }
    .await;

    match result {
        Ok(recording) => {
            // Broadcast the removal of the untracked recording
            let clients = state.ws_clients.clone();
            tokio::spawn(async move {
                broadcast_untracked_removed(&clients, id).await;
            });
            (StatusCode::CREATED, Json(recording)).into_response()
        }
        Err(e) if e.to_string() == "NOT_FOUND" => StatusCode::NOT_FOUND.into_response(),
        Err(e) if e.to_string() == "EVENT_NOT_FOUND" => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("assign_untracked_recording: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct DeleteUntrackedParams {
    #[serde(default)]
    pub delete_file: bool,
}

pub async fn delete_untracked_recording(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<DeleteUntrackedParams>,
) -> impl IntoResponse {
    let row = sqlx::query_as::<_, untracked_recording::UntrackedRecording>(
        "SELECT * FROM untracked_recordings WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some(rec)) => {
            let del = sqlx::query("DELETE FROM untracked_recordings WHERE id = $1")
                .bind(id)
                .execute(&state.pool)
                .await;
            if let Err(e) = del {
                tracing::error!("delete_untracked_recording DB: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            if params.delete_file {
                if let Err(e) = tokio::fs::remove_file(&rec.file_path).await {
                    tracing::warn!("delete_untracked_recording: could not delete file {}: {e}", rec.file_path);
                }
            }
            let clients = state.ws_clients.clone();
            tokio::spawn(async move {
                broadcast_untracked_removed(&clients, id).await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("delete_untracked_recording fetch: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ── Event activities ───────────────────────────────────────────────────────────

pub async fn list_event_activities(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> impl IntoResponse {
    match activity::list_activities(event_id, &state.pool).await {
        Ok(activities) => (StatusCode::OK, Json(activities)).into_response(),
        Err(e) => {
            tracing::error!("list_event_activities: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn create_event_activity(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
    Json(body): Json<CreateEventActivity>,
) -> impl IntoResponse {
    // Verify event exists
    let event_exists: Result<bool, _> =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM events WHERE id = $1)")
            .bind(event_id)
            .fetch_one(&state.pool)
            .await;

    match event_exists {
        Ok(false) | Err(_) => return StatusCode::NOT_FOUND.into_response(),
        Ok(true) => {}
    }

    let result = sqlx::query_as::<_, activity::EventActivity>(
        "INSERT INTO event_activities (event_id, activity_type, message) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(event_id)
    .bind(&body.activity_type)
    .bind(&body.message)
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(act) => (StatusCode::CREATED, Json(act)).into_response(),
        Err(e) => {
            tracing::error!("create_event_activity: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn delete_event_activity(
    State(state): State<AppState>,
    Path((event_id, activity_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let result = sqlx::query(
        "DELETE FROM event_activities WHERE id = $1 AND event_id = $2 RETURNING id",
    )
    .bind(activity_id)
    .bind(event_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(_)) => StatusCode::NO_CONTENT.into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("delete_event_activity: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ── Broadlink ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BroadlinkDevice {
    id: Uuid,
    name: String,
    device_type: String,
    model: Option<String>,
    host: String,
    mac: String,
    is_default: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BroadlinkCommand {
    id: Uuid,
    device_id: Option<Uuid>,
    name: String,
    slug: String,
    code: String,
    code_type: String,
    category: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDeviceBody {
    name: String,
    host: String,
    mac: String,
    device_type: String,
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCommandBody {
    device_id: Option<Uuid>,
    name: String,
    slug: String,
    code: String,
    code_type: String,
    category: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommandBody {
    name: Option<String>,
    slug: Option<String>,
    code: Option<String>,
    code_type: Option<String>,
    category: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearnBody {
    signal_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CommandsQuery {
    device_id: Option<Uuid>,
    category: Option<String>,
}

pub async fn broadlink_get_status(State(state): State<AppState>) -> impl IntoResponse {
    let status = state.broadlink_connector.get_status().await;
    Json(json!({ "status": status }))
}

pub async fn broadlink_list_devices(State(state): State<AppState>) -> impl IntoResponse {
    let rows = sqlx::query_as::<_, (Uuid, String, String, Option<String>, String, String, bool)>(
        "SELECT id, name, device_type, model, host, mac, is_default FROM broadlink_devices ORDER BY created_at",
    )
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(devices) => {
            let list: Vec<BroadlinkDevice> = devices
                .into_iter()
                .map(|(id, name, device_type, model, host, mac, is_default)| BroadlinkDevice {
                    id,
                    name,
                    device_type,
                    model,
                    host,
                    mac,
                    is_default,
                })
                .collect();
            Json(list).into_response()
        }
        Err(e) => {
            tracing::error!("broadlink_list_devices: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn broadlink_add_device(
    State(state): State<AppState>,
    Json(body): Json<AddDeviceBody>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, (Uuid,)>(
        "INSERT INTO broadlink_devices (name, device_type, model, host, mac) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(&body.name)
    .bind(&body.device_type)
    .bind(&body.model)
    .bind(&body.host)
    .bind(&body.mac)
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok((id,)) => {
            // Update connector status: now we have at least one device
            state
                .broadlink_connector
                .set_status(crate::connectors::ConnectorStatus::Connected)
                .await;
            let device = BroadlinkDevice {
                id,
                name: body.name,
                device_type: body.device_type,
                model: body.model,
                host: body.host,
                mac: body.mac,
                is_default: false,
            };
            (StatusCode::CREATED, Json(device)).into_response()
        }
        Err(e) => {
            tracing::error!("broadlink_add_device: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn broadlink_remove_device(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM broadlink_devices WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => StatusCode::NOT_FOUND.into_response(),
        Ok(_) => {
            // Check if any devices remain; update status accordingly
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM broadlink_devices")
                .fetch_one(&state.pool)
                .await
                .unwrap_or(0);
            let new_status = if count > 0 {
                crate::connectors::ConnectorStatus::Connected
            } else {
                crate::connectors::ConnectorStatus::Disconnected
            };
            state.broadlink_connector.set_status(new_status).await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!("broadlink_remove_device: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn broadlink_discover(State(state): State<AppState>) -> impl IntoResponse {
    let clients = state.ws_clients.clone();
    let pool = state.pool.clone();
    let connector = state.broadlink_connector.clone();

    tokio::spawn(async move {
        match crate::broadlink::discover_devices(5).await {
            Ok(devices) => {
                for dev in devices {
                    let msg = json!({
                        "type": "broadlink.device.discovered",
                        "device": {
                            "name": dev.name,
                            "host": dev.host,
                            "mac": dev.mac,
                            "deviceType": dev.device_type,
                            "model": dev.model,
                        }
                    })
                    .to_string();
                    let guard = clients.read().await;
                    for tx in guard.values() {
                        let _ = tx.send(axum::extract::ws::Message::Text(msg.clone().into()));
                    }
                    drop(guard);

                    // Upsert discovered device into DB
                    let _ = sqlx::query(
                        "INSERT INTO broadlink_devices (name, device_type, model, host, mac, last_seen_at) \
                         VALUES ($1, $2, $3, $4, $5, NOW()) \
                         ON CONFLICT (mac) DO UPDATE SET host = EXCLUDED.host, last_seen_at = NOW()",
                    )
                    .bind(&dev.name)
                    .bind(&dev.device_type)
                    .bind(&dev.model)
                    .bind(&dev.host)
                    .bind(&dev.mac)
                    .execute(&pool)
                    .await;
                }

                // Refresh status after upserts
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM broadlink_devices")
                    .fetch_one(&pool)
                    .await
                    .unwrap_or(0);
                let new_status = if count > 0 {
                    crate::connectors::ConnectorStatus::Connected
                } else {
                    crate::connectors::ConnectorStatus::Disconnected
                };
                connector.set_status(new_status).await;
            }
            Err(e) => tracing::error!("broadlink_discover: {e}"),
        }
    });

    StatusCode::ACCEPTED.into_response()
}

pub async fn broadlink_list_commands(
    State(state): State<AppState>,
    Query(q): Query<CommandsQuery>,
) -> impl IntoResponse {
    let rows = if let Some(device_id) = q.device_id {
        if let Some(cat) = q.category {
            sqlx::query_as::<_, (Uuid, Option<Uuid>, String, String, String, String, String)>(
                "SELECT id, device_id, name, slug, code, code_type, category \
                 FROM broadlink_commands WHERE device_id = $1 AND category = $2 ORDER BY created_at",
            )
            .bind(device_id)
            .bind(cat)
            .fetch_all(&state.pool)
            .await
        } else {
            sqlx::query_as::<_, (Uuid, Option<Uuid>, String, String, String, String, String)>(
                "SELECT id, device_id, name, slug, code, code_type, category \
                 FROM broadlink_commands WHERE device_id = $1 ORDER BY created_at",
            )
            .bind(device_id)
            .fetch_all(&state.pool)
            .await
        }
    } else {
        sqlx::query_as::<_, (Uuid, Option<Uuid>, String, String, String, String, String)>(
            "SELECT id, device_id, name, slug, code, code_type, category \
             FROM broadlink_commands ORDER BY created_at",
        )
        .fetch_all(&state.pool)
        .await
    };

    match rows {
        Ok(commands) => {
            let list: Vec<BroadlinkCommand> = commands
                .into_iter()
                .map(|(id, device_id, name, slug, code, code_type, category)| BroadlinkCommand {
                    id,
                    device_id,
                    name,
                    slug,
                    code,
                    code_type,
                    category,
                })
                .collect();
            Json(list).into_response()
        }
        Err(e) => {
            tracing::error!("broadlink_list_commands: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn broadlink_add_command(
    State(state): State<AppState>,
    Json(body): Json<AddCommandBody>,
) -> impl IntoResponse {
    let category = body.category.unwrap_or_else(|| "other".to_string());
    let result = sqlx::query_as::<_, (Uuid,)>(
        "INSERT INTO broadlink_commands (device_id, name, slug, code, code_type, category) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(body.device_id)
    .bind(&body.name)
    .bind(&body.slug)
    .bind(&body.code)
    .bind(&body.code_type)
    .bind(&category)
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok((id,)) => {
            let cmd = BroadlinkCommand {
                id,
                device_id: body.device_id,
                name: body.name,
                slug: body.slug,
                code: body.code,
                code_type: body.code_type,
                category,
            };
            (StatusCode::CREATED, Json(cmd)).into_response()
        }
        Err(e) => {
            tracing::error!("broadlink_add_command: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn broadlink_update_command(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateCommandBody>,
) -> impl IntoResponse {
    let result = sqlx::query(
        "UPDATE broadlink_commands SET \
         name = COALESCE($2, name), \
         slug = COALESCE($3, slug), \
         code = COALESCE($4, code), \
         code_type = COALESCE($5, code_type), \
         category = COALESCE($6, category), \
         updated_at = NOW() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(&body.name)
    .bind(&body.slug)
    .bind(&body.code)
    .bind(&body.code_type)
    .bind(&body.category)
    .execute(&state.pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => StatusCode::NOT_FOUND.into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("broadlink_update_command: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn broadlink_remove_command(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM broadlink_commands WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => StatusCode::NOT_FOUND.into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("broadlink_remove_command: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn broadlink_start_learn(
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
    Json(body): Json<Option<LearnBody>>,
) -> impl IntoResponse {
    // Prevent concurrent learns
    if state
        .broadlink_learn_active
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "error": "Learning already in progress" })),
        )
            .into_response();
    }

    let signal_type = body
        .and_then(|b| b.signal_type)
        .unwrap_or_else(|| "ir".to_string());

    // Fetch device info
    let device = sqlx::query_as::<_, (String, String, String)>(
        "SELECT host, mac, device_type FROM broadlink_devices WHERE id = $1",
    )
    .bind(device_id)
    .fetch_optional(&state.pool)
    .await;

    let (host, mac, devtype) = match device {
        Ok(Some(row)) => row,
        Ok(None) => {
            state
                .broadlink_learn_active
                .store(false, Ordering::SeqCst);
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Device not found" })),
            )
                .into_response();
        }
        Err(e) => {
            state
                .broadlink_learn_active
                .store(false, Ordering::SeqCst);
            tracing::error!("broadlink_start_learn fetch device: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let learn_active = state.broadlink_learn_active.clone();
    let learn_tx = state.broadlink_connector.learn_tx.clone();

    tokio::spawn(async move {
        let result =
            crate::broadlink::learn_code(&host, &mac, &devtype, &signal_type).await;
        let event = match result {
            Ok(lr) => crate::connectors::broadlink::BroadlinkLearnEvent {
                code: lr.code,
                error: lr.error,
            },
            Err(e) => crate::connectors::broadlink::BroadlinkLearnEvent {
                code: None,
                error: Some(e),
            },
        };
        let _ = learn_tx.send(event);
        learn_active.store(false, Ordering::SeqCst);
    });

    StatusCode::ACCEPTED.into_response()
}

pub async fn broadlink_cancel_learn(State(state): State<AppState>) -> impl IntoResponse {
    crate::broadlink::cancel_learn().await;
    state
        .broadlink_learn_active
        .store(false, Ordering::SeqCst);
    StatusCode::NO_CONTENT.into_response()
}

pub async fn broadlink_send_command(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let row = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT bc.code, bd.host, bd.mac, bd.device_type \
         FROM broadlink_commands bc \
         JOIN broadlink_devices bd ON bc.device_id = bd.id \
         WHERE bc.id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await;

    let (code, host, mac, devtype) = match row {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Command not found" })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("broadlink_send_command fetch: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match crate::broadlink::send_code(&host, &mac, &devtype, &code).await {
        Ok(r) if r.success => StatusCode::NO_CONTENT.into_response(),
        Ok(r) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": r.error.unwrap_or_default() })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("broadlink_send_command send: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
