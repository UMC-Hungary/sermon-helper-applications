use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};

// ── Message types from the server ────────────────────────────────────────────

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SlideContent {
    pub index: u32,
    pub texts: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PresenterState {
    pub current_slide: u32,
    pub slides: Vec<SlideContent>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum ServerMsg {
    #[serde(rename = "presenter.state")]
    PresenterState { state: PresenterState },
    #[serde(rename = "presenter.slide_changed")]
    SlideChanged {
        #[serde(rename = "currentSlide")]
        current_slide: u32,
    },
    #[serde(rename = "ping")]
    Ping {
        #[serde(rename = "pingId")]
        ping_id: i64,
    },
    #[serde(other)]
    Unknown,
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Connect to the presenter WebSocket, render slides, and send pixel frames
/// down `tx` whenever the current slide changes. Reconnects automatically.
pub async fn run(url: String, tx: std::sync::mpsc::Sender<Frame>, dims: DisplayDims) {
    loop {
        eprintln!("[ws] Connecting to {url}...");
        match connect_and_receive(&url, &tx, dims).await {
            Ok(()) => eprintln!("[ws] Connection closed."),
            Err(e) => eprintln!("[ws] Error: {e}"),
        }
        eprintln!("[ws] Reconnecting in 3 s...");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

// ── Frame type sent over the channel ─────────────────────────────────────────

/// Rendered pixel frame: u32 values in 0x00RRGGBB order (minifb / fb compatible).
pub type Frame = Vec<u32>;

/// Display dimensions needed by the renderer.
#[derive(Clone, Copy)]
pub struct DisplayDims {
    pub width: u32,
    pub height: u32,
}

// ── Internal connection loop ──────────────────────────────────────────────────

async fn connect_and_receive(
    url: &str,
    tx: &std::sync::mpsc::Sender<Frame>,
    dims: DisplayDims,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (ws_stream, _) = connect_async(url).await?;
    eprintln!("[ws] Connected.");

    let (mut write, mut read) = ws_stream.split();

    // Announce ourselves and ask for current state immediately
    write
        .send(Message::Text(
            json!({"type": "presenter.register", "label": "Presenter Receiver"})
                .to_string()
                .into(),
        ))
        .await?;
    write
        .send(Message::Text(
            json!({"type": "presenter.status"}).to_string().into(),
        ))
        .await?;

    let mut slides: Vec<SlideContent> = Vec::new();
    let mut current_slide: u32;

    while let Some(raw) = read.next().await {
        let raw = raw?;
        let text = match raw {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };

        let msg: ServerMsg = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(_) => continue,
        };

        match msg {
            ServerMsg::PresenterState { state } => {
                current_slide = state.current_slide;
                slides = state.slides;
                send_frame(&slides, current_slide, tx, dims);
            }
            ServerMsg::SlideChanged { current_slide: new_slide } => {
                current_slide = new_slide;
                send_frame(&slides, current_slide, tx, dims);
            }
            ServerMsg::Ping { ping_id } => {
                write
                    .send(Message::Text(
                        json!({"type": "pong", "ping_id": ping_id})
                            .to_string()
                            .into(),
                    ))
                    .await?;
            }
            ServerMsg::Unknown => {}
        }
    }

    Ok(())
}

// ── Render + send ─────────────────────────────────────────────────────────────

fn send_frame(
    slides: &[SlideContent],
    current: u32,
    tx: &std::sync::mpsc::Sender<Frame>,
    dims: DisplayDims,
) {
    let texts: &[String] = slides
        .iter()
        .find(|s| s.index == current)
        .map(|s| s.texts.as_slice())
        .unwrap_or(&[]);

    let rgb = crate::renderer::render_slide(texts, dims.width, dims.height);
    let _ = tx.send(crate::renderer::rgb_to_u32(&rgb));
}
