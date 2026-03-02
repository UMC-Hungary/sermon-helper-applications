use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::connectors::{facebook, youtube};
use crate::models::{
    cron_job::{self, CreateCronJob, UpdateCronJob},
    event::{fetch_event, CreateEvent, EventSummary, UpdateEvent},
    recording::{CreateRecording, Recording},
};
use crate::server::websocket::{broadcast_event_changed, spawn_scheduling_tasks};
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
                    state.youtube_connector.start(state.pool.clone(), config, state.app_handle.clone()).await;
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
                    state.facebook_connector.start(state.pool.clone(), state.app_handle.clone()).await;
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


// ── Stream stats (mediamtx API proxy) ────────────────────────────────────────

/// Proxy the mediamtx `/v3/paths/list` response to a minimal stats object.
/// Returns a fixed shape even when mediamtx is not running (ready: false).
pub async fn get_stream_stats() -> impl IntoResponse {
    #[derive(Deserialize)]
    struct MtxPath {
        ready: bool,
        #[serde(rename = "bytesReceived")]
        bytes_received: u64,
        #[serde(rename = "bytesSent")]
        bytes_sent: u64,
        tracks: Vec<String>,
        readers: Vec<serde_json::Value>,
    }

    #[derive(Deserialize)]
    struct MtxList {
        items: Vec<MtxPath>,
    }

    let client = reqwest::Client::new();
    let url = format!("http://localhost:{}/v3/paths/list", crate::mediamtx::API_PORT);

    let result = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await;

    let offline = json!({
        "ready": false,
        "bytesReceived": 0u64,
        "bytesSent": 0u64,
        "readers": 0u32,
        "tracks": serde_json::Value::Array(vec![]),
    });

    match result {
        Ok(r) if r.status().is_success() => match r.json::<MtxList>().await {
            Ok(list) => {
                let live = list.items.into_iter().find(|p| p.ready);
                match live {
                    Some(p) => Json(json!({
                        "ready": true,
                        "bytesReceived": p.bytes_received,
                        "bytesSent": p.bytes_sent,
                        "readers": p.readers.len() as u32,
                        "tracks": p.tracks,
                    }))
                    .into_response(),
                    None => Json(offline).into_response(),
                }
            }
            Err(_) => Json(offline).into_response(),
        },
        _ => Json(offline).into_response(),
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
    match fetch_event(id, &state.pool).await {
        Ok(Some(event)) => (StatusCode::OK, Json(event)).into_response(),
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
    let result: anyhow::Result<_> = async {
        let mut tx = state.pool.begin().await?;
        sqlx::query("SET LOCAL app.skip_sync_notify = 'true'")
            .execute(&mut *tx)
            .await?;

        let event_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO events (
                title, date_time, speaker, description, textus, leckio,
                textus_translation, leckio_translation, auto_upload_enabled
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id"#,
        )
        .bind(&body.title)
        .bind(body.date_time)
        .bind(body.speaker.unwrap_or_default())
        .bind(body.description.unwrap_or_default())
        .bind(body.textus.unwrap_or_default())
        .bind(body.leckio.unwrap_or_default())
        .bind(body.textus_translation.unwrap_or_else(|| "UF".to_string()))
        .bind(body.leckio_translation.unwrap_or_else(|| "UF".to_string()))
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
                textus = $5,
                leckio = $6,
                textus_translation = $7,
                leckio_translation = $8,
                auto_upload_enabled = $9,
                updated_at = NOW()
            WHERE id = $10
            RETURNING id"#,
        )
        .bind(&body.title)
        .bind(body.date_time)
        .bind(body.speaker.unwrap_or_default())
        .bind(body.description.unwrap_or_default())
        .bind(body.textus.unwrap_or_default())
        .bind(body.leckio.unwrap_or_default())
        .bind(body.textus_translation.unwrap_or_else(|| "UF".to_string()))
        .bind(body.leckio_translation.unwrap_or_else(|| "UF".to_string()))
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

        cron_job::sync_features(&mut tx, row.0, body.pull_youtube).await?;
        tx.commit().await?;

        Ok(cron_job::CronJob {
            id: row.0,
            name: row.1,
            cron_expression: row.2,
            enabled: row.3,
            pull_youtube: body.pull_youtube,
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
            tokio::spawn(async move {
                sched.reload(pool, clients, yt).await;
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

        cron_job::sync_features(&mut tx, row.0, body.pull_youtube).await?;
        tx.commit().await?;

        Ok(Some(cron_job::CronJob {
            id: row.0,
            name: row.1,
            cron_expression: row.2,
            enabled: row.3,
            pull_youtube: body.pull_youtube,
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
            tokio::spawn(async move {
                sched.reload(pool, clients, yt).await;
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
            tokio::spawn(async move {
                sched.reload(pool, clients, yt).await;
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
