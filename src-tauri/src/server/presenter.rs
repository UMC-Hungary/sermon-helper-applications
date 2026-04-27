use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Read;

use crate::server::AppState;

// ── Data types ────────────────────────────────────────────────────────────────

/// A single paragraph extracted from a slide, including its text and alignment.
///
/// `text` may contain `\n` characters from `<a:br>` hard line breaks within
/// the paragraph. `align` is one of `"left"`, `"center"`, `"right"`, or
/// `"justify"` (mapped from the PPTX `algn` attribute).
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphContent {
    pub text: String,
    pub align: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SlideContent {
    pub index: u32,
    pub paragraphs: Vec<ParagraphContent>,
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

/// Map a raw PPTX `algn` attribute value to a CSS text-align keyword.
fn map_align(raw: &[u8]) -> &'static str {
    match raw {
        b"ctr" => "center",
        b"r" => "right",
        b"just" | b"dist" => "justify",
        _ => "left",
    }
}

/// Parse all visible text from a single slide's XML content.
///
/// Each `<a:p>` becomes a `ParagraphContent`. The paragraph's `align` is read
/// from the `algn` attribute of its `<a:pPr>` element (defaulting to `"left"`).
/// Text runs (`<a:t>`) within a paragraph are concatenated; `<a:br>` inserts
/// a `\n` within the paragraph string.
fn parse_slide_xml(xml: &[u8]) -> Vec<ParagraphContent> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_reader(xml);
    let mut buf: Vec<u8> = Vec::new();
    let mut paragraphs: Vec<ParagraphContent> = Vec::new();
    let mut current_para = String::new();
    let mut current_align = "left";
    let mut in_text_run = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().into_inner() {
                b"a:p" => {
                    current_para.clear();
                    current_align = "left";
                }
                b"a:pPr" => {
                    // Paragraph properties — read alignment attribute.
                    for attr in e.attributes().flatten() {
                        if attr.key.into_inner() == b"algn" {
                            current_align = map_align(attr.value.as_ref());
                            break;
                        }
                    }
                }
                b"a:t" => in_text_run = true,
                _ => {}
            },
            Ok(Event::Empty(ref e)) => match e.name().into_inner() {
                b"a:br" => {
                    // Hard line break within a paragraph.
                    current_para.push('\n');
                }
                b"a:pPr" => {
                    // Self-closing paragraph properties — read alignment attribute.
                    for attr in e.attributes().flatten() {
                        if attr.key.into_inner() == b"algn" {
                            current_align = map_align(attr.value.as_ref());
                            break;
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) => {
                if in_text_run {
                    if let Ok(decoded) = e.decode() {
                        let unescaped = quick_xml::escape::unescape(&decoded)
                            .map(|c| c.into_owned())
                            .unwrap_or_else(|_| decoded.into_owned());
                        current_para.push_str(&unescaped);
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.name().into_inner() {
                b"a:t" => in_text_run = false,
                b"a:p" => {
                    let trimmed = current_para.trim().to_string();
                    if !trimmed.is_empty() {
                        paragraphs.push(ParagraphContent {
                            text: trimmed,
                            align: current_align.to_string(),
                        });
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
            paragraphs: parse_slide_xml(&xml_bytes),
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
    pub muted: bool,
}

impl PresenterState {
    pub fn empty() -> Self {
        Self {
            loaded: false,
            file_path: None,
            current_slide: 0,
            total_slides: 0,
            slides: Vec::new(),
            muted: false,
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
            muted: false,
        }
    }

    pub fn mute(&mut self) {
        self.muted = true;
    }

    pub fn unmute(&mut self) {
        self.muted = false;
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

    /// Update the text of a slide identified by its 1-based index.
    ///
    /// Receives plain text lines from the editor and maps them back to
    /// `ParagraphContent`, preserving the alignment of each matching original
    /// paragraph. Extra lines (beyond the original paragraph count) receive
    /// `"left"` alignment.
    ///
    /// Does nothing when not loaded or when `slide_index` is out of range.
    pub fn update_slide(&mut self, slide_index: u32, texts: Vec<String>) {
        if !self.loaded {
            return;
        }
        if let Some(slide) = self.slides.iter_mut().find(|s| s.index == slide_index) {
            let old = std::mem::take(&mut slide.paragraphs);
            slide.paragraphs = texts
                .into_iter()
                .enumerate()
                .map(|(i, text)| {
                    let align = old
                        .get(i)
                        .map(|p| p.align.as_str())
                        .unwrap_or("left")
                        .to_string();
                    ParagraphContent { text, align }
                })
                .collect();
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
