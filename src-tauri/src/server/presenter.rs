use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Read;

use crate::server::AppState;

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SlideContent {
    pub index: u32,
    pub texts: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedPresentation {
    pub file_path: String,
    pub total_slides: u32,
    pub slides: Vec<SlideContent>,
}

// ── PPTX parsing ──────────────────────────────────────────────────────────────

/// Extract the 1-based slide number from a filename like "slide3.xml" → Some(3).
fn slide_number(basename: &str) -> Option<u32> {
    basename
        .strip_suffix(".xml")
        .and_then(|s| s.strip_prefix("slide"))
        .and_then(|n| n.parse::<u32>().ok())
}

/// Parse all visible text from a single slide's XML content.
///
/// Text runs (`<a:t>`) within a paragraph (`<a:p>`) are joined together.
/// Each non-empty paragraph is returned as one string.
fn parse_slide_xml(xml: &[u8]) -> Vec<String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_reader(xml);
    let mut buf: Vec<u8> = Vec::new();
    let mut paragraphs: Vec<String> = Vec::new();
    let mut current_para = String::new();
    let mut in_text_run = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().into_inner() {
                b"a:t" => in_text_run = true,
                b"a:p" => current_para.clear(),
                _ => {}
            },
            Ok(Event::Text(e)) => {
                if in_text_run {
                    if let Ok(decoded) = e.decode() {
                        let unescaped = quick_xml::escape::unescape(&decoded)
                            .map(|c| c.into_owned())
                            .unwrap_or_else(|_| decoded.into_owned());
                        current_para.push_str(&unescaped);
                    }
                }
            }
            Ok(Event::End(e)) => match e.name().into_inner() {
                b"a:t" => in_text_run = false,
                b"a:p" => {
                    let trimmed = current_para.trim().to_string();
                    if !trimmed.is_empty() {
                        paragraphs.push(trimmed);
                    }
                    current_para.clear();
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    paragraphs
}

/// Parse a `.pptx` file and return structured slide content.
///
/// Only `.pptx` (Open XML) format is supported — the legacy binary `.ppt`
/// format is not parseable without proprietary libraries.
pub fn parse_pptx(file_path: &str) -> Result<ParsedPresentation, String> {
    let file =
        std::fs::File::open(file_path).map_err(|e| format!("Cannot open file: {e}"))?;

    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Not a valid .pptx file: {e}"))?;

    // Collect all slide entry names from "ppt/slides/slideN.xml".
    let mut slide_names: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            let name = archive.by_index(i).ok()?.name().to_string();
            let basename = name.strip_prefix("ppt/slides/")?;
            // Exclude nested paths (e.g. _rels/slide1.xml.rels).
            if basename.contains('/') {
                return None;
            }
            slide_number(basename)?; // ensure parseable number
            Some(name)
        })
        .collect();

    if slide_names.is_empty() {
        return Err(
            "No slides found. Only .pptx format is supported — please re-save .ppt files as .pptx."
                .to_string(),
        );
    }

    // Sort by slide number so the array is in presentation order.
    slide_names.sort_by_key(|name| {
        name.strip_prefix("ppt/slides/")
            .and_then(slide_number)
            .unwrap_or(0)
    });

    let total = slide_names.len() as u32;
    let mut slides: Vec<SlideContent> = Vec::with_capacity(slide_names.len());

    for (idx, name) in slide_names.iter().enumerate() {
        let mut xml_bytes: Vec<u8> = Vec::new();
        archive
            .by_name(name)
            .map_err(|e| format!("Cannot open slide entry '{name}': {e}"))?
            .read_to_end(&mut xml_bytes)
            .map_err(|e| format!("Cannot read slide content: {e}"))?;

        slides.push(SlideContent {
            index: idx as u32 + 1,
            texts: parse_slide_xml(&xml_bytes),
        });
    }

    Ok(ParsedPresentation {
        file_path: file_path.to_string(),
        total_slides: total,
        slides,
    })
}

// ── Live presenter state ──────────────────────────────────────────────────────

/// In-memory state for the active web-presenter session.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenterState {
    pub loaded: bool,
    pub file_path: Option<String>,
    pub current_slide: u32,
    pub total_slides: u32,
    pub slides: Vec<SlideContent>,
}

impl PresenterState {
    pub fn empty() -> Self {
        Self {
            loaded: false,
            file_path: None,
            current_slide: 0,
            total_slides: 0,
            slides: Vec::new(),
        }
    }

    pub fn from_parsed(parsed: ParsedPresentation) -> Self {
        let total = parsed.total_slides;
        Self {
            loaded: true,
            file_path: Some(parsed.file_path),
            current_slide: if total > 0 { 1 } else { 0 },
            total_slides: total,
            slides: parsed.slides,
        }
    }

    /// Advance one slide forward; clamps at `total_slides`.
    pub fn go_next(&mut self) {
        if self.loaded && self.current_slide < self.total_slides {
            self.current_slide += 1;
        }
    }

    /// Go back one slide; clamps at 1.
    pub fn go_prev(&mut self) {
        if self.loaded && self.current_slide > 1 {
            self.current_slide -= 1;
        }
    }

    /// Jump to the first slide.
    pub fn go_first(&mut self) {
        if self.loaded && self.total_slides > 0 {
            self.current_slide = 1;
        }
    }

    /// Jump to the last slide.
    pub fn go_last(&mut self) {
        if self.loaded {
            self.current_slide = self.total_slides;
        }
    }

    /// Jump to a specific 1-based slide number, clamped to valid range.
    pub fn go_to(&mut self, slide: u32) {
        if self.loaded && self.total_slides > 0 {
            self.current_slide = slide.max(1).min(self.total_slides);
        }
    }
}

// ── HTTP handler ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseBody {
    pub file_path: String,
}

/// `POST /api/presenter/parse`
///
/// Body: `{ "filePath": "/absolute/path/to/file.pptx" }`
///
/// Returns the full parsed presentation as JSON so callers can verify the
/// extracted content before the full web-presenter feature is wired up.
pub async fn parse_presentation(
    State(_state): State<AppState>,
    Json(body): Json<ParseBody>,
) -> impl IntoResponse {
    let file_path = body.file_path;
    let result = tokio::task::spawn_blocking(move || parse_pptx(&file_path)).await;

    match result {
        Ok(Ok(parsed)) => (
            StatusCode::OK,
            Json(json!({ "success": true, "data": parsed })),
        ),
        Ok(Err(e)) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "success": false, "error": e })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}
