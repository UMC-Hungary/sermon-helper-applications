use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Read;

use crate::server::AppState;

// ── Data types ────────────────────────────────────────────────────────────────

/// A single paragraph from a slide, with each visual line stored separately.
///
/// `lines` contains the text of each visual line; `<a:br>` (Shift+Enter) in
/// the PPTX produces one entry per break.  `align` is a CSS keyword.
/// `font_size_pt` is the author-specified size in points (0.0 = not found).
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphContent {
    pub lines: Vec<String>,
    pub align: String,
    pub font_size_pt: f32,
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
    /// Slide width in EMUs (English Metric Units; 914 400 EMU = 1 inch).
    pub slide_width_emu: u64,
    /// Slide height in EMUs.
    pub slide_height_emu: u64,
}

// ── PPTX parsing ──────────────────────────────────────────────────────────────

fn slide_number(basename: &str) -> Option<u32> {
    basename
        .strip_suffix(".xml")
        .and_then(|s| s.strip_prefix("slide"))
        .and_then(|n| n.parse::<u32>().ok())
}

fn map_align(raw: &[u8]) -> &'static str {
    match raw {
        b"ctr" => "center",
        b"r" => "right",
        b"just" | b"dist" => "justify",
        _ => "left",
    }
}

/// Read the `sz` attribute (centipoints) from a run-properties element and
/// convert to points.  Returns 0.0 if the attribute is absent or unparseable.
fn read_sz(e: &quick_xml::events::BytesStart<'_>) -> f32 {
    for attr in e.attributes().flatten() {
        if attr.key.into_inner() == b"sz" {
            if let Ok(s) = std::str::from_utf8(attr.value.as_ref()) {
                if let Ok(n) = s.parse::<u32>() {
                    return n as f32 / 100.0;
                }
            }
        }
    }
    0.0
}

/// Trim `current` and push it to `lines` if non-empty; then clear `current`.
fn push_line(lines: &mut Vec<String>, current: &mut String) {
    let trimmed = current.trim().to_string();
    current.clear();
    if !trimmed.is_empty() {
        lines.push(trimmed);
    }
}

/// Parse all visible text from a single slide's XML.
///
/// Each `<a:p>` becomes a `ParagraphContent`.  `<a:br>` (Shift+Enter in PPT)
/// is honoured as an explicit visual line break within the paragraph — these
/// are always intentional author decisions, never automatic word-wrap artefacts.
/// The paragraph font size is taken from the first run/default-run properties
/// element (`<a:rPr>`, `<a:defRPr>`, `<a:endParaRPr>`) that carries an `sz`
/// attribute.
fn parse_slide_xml(xml: &[u8]) -> Vec<ParagraphContent> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_reader(xml);
    let mut buf: Vec<u8> = Vec::new();
    let mut paragraphs: Vec<ParagraphContent> = Vec::new();

    let mut para_lines: Vec<String> = Vec::new();
    let mut current_line = String::new();
    let mut current_align = "left";
    let mut font_size_pt: f32 = 0.0;
    let mut in_text_run = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().into_inner() {
                b"a:p" => {
                    para_lines.clear();
                    current_line.clear();
                    current_align = "left";
                    font_size_pt = 0.0;
                }
                b"a:pPr" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.into_inner() == b"algn" {
                            current_align = map_align(attr.value.as_ref());
                            break;
                        }
                    }
                }
                b"a:rPr" | b"a:defRPr" | b"a:endParaRPr" => {
                    if font_size_pt == 0.0 {
                        font_size_pt = read_sz(e);
                    }
                }
                b"a:br" => {
                    push_line(&mut para_lines, &mut current_line);
                }
                b"a:t" => in_text_run = true,
                _ => {}
            },
            Ok(Event::Empty(ref e)) => match e.name().into_inner() {
                b"a:pPr" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.into_inner() == b"algn" {
                            current_align = map_align(attr.value.as_ref());
                            break;
                        }
                    }
                }
                b"a:rPr" | b"a:defRPr" | b"a:endParaRPr" => {
                    if font_size_pt == 0.0 {
                        font_size_pt = read_sz(e);
                    }
                }
                b"a:br" => {
                    push_line(&mut para_lines, &mut current_line);
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) => {
                if in_text_run {
                    if let Ok(decoded) = e.decode() {
                        let unescaped = quick_xml::escape::unescape(&decoded)
                            .map(|c| c.into_owned())
                            .unwrap_or_else(|_| decoded.into_owned());
                        current_line.push_str(&unescaped);
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.name().into_inner() {
                b"a:t" => in_text_run = false,
                b"a:p" => {
                    push_line(&mut para_lines, &mut current_line);
                    if !para_lines.is_empty() {
                        paragraphs.push(ParagraphContent {
                            lines: std::mem::take(&mut para_lines),
                            align: current_align.to_string(),
                            font_size_pt,
                        });
                    }
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

/// Parse `ppt/presentation.xml` for the `<p:sldSz>` element and return its
/// `cx`/`cy` attributes in EMUs.  Falls back to the standard 16:9 dimensions.
fn parse_slide_size(xml: &[u8]) -> (u64, u64) {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_reader(xml);
    let mut buf: Vec<u8> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                if e.name().into_inner() == b"p:sldSz" {
                    let mut cx = 0u64;
                    let mut cy = 0u64;
                    for attr in e.attributes().flatten() {
                        match attr.key.into_inner() {
                            b"cx" => {
                                if let Ok(s) = std::str::from_utf8(attr.value.as_ref()) {
                                    cx = s.parse().unwrap_or(0);
                                }
                            }
                            b"cy" => {
                                if let Ok(s) = std::str::from_utf8(attr.value.as_ref()) {
                                    cy = s.parse().unwrap_or(0);
                                }
                            }
                            _ => {}
                        }
                    }
                    if cx > 0 && cy > 0 {
                        return (cx, cy);
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    (12192000, 6858000) // default 16:9 widescreen
}

/// Parse a `.pptx` file and return structured slide content.
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
            if basename.contains('/') {
                return None;
            }
            slide_number(basename)?;
            Some(name)
        })
        .collect();

    if slide_names.is_empty() {
        return Err(
            "No slides found. Only .pptx format is supported — please re-save .ppt files as .pptx."
                .to_string(),
        );
    }

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

    // Read slide dimensions after all slides — avoids any zip seek-order issue.
    let (slide_width_emu, slide_height_emu) =
        if let Ok(mut entry) = archive.by_name("ppt/presentation.xml") {
            let mut xml_bytes: Vec<u8> = Vec::new();
            match entry.read_to_end(&mut xml_bytes) {
                Ok(_) => parse_slide_size(&xml_bytes),
                Err(_) => (12192000u64, 6858000u64),
            }
        } else {
            (12192000u64, 6858000u64)
        };

    Ok(ParsedPresentation {
        file_path: file_path.to_string(),
        total_slides: total,
        slides,
        slide_width_emu,
        slide_height_emu,
    })
}

// ── Live presenter state ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenterState {
    pub loaded: bool,
    pub file_path: Option<String>,
    pub current_slide: u32,
    pub total_slides: u32,
    pub slides: Vec<SlideContent>,
    pub muted: bool,
    pub slide_width_emu: u64,
    pub slide_height_emu: u64,
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
            slide_width_emu: 12192000,
            slide_height_emu: 6858000,
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
            slide_width_emu: parsed.slide_width_emu,
            slide_height_emu: parsed.slide_height_emu,
        }
    }

    pub fn mute(&mut self) {
        self.muted = true;
    }

    pub fn unmute(&mut self) {
        self.muted = false;
    }

    pub fn go_next(&mut self) {
        if self.loaded && self.current_slide < self.total_slides {
            self.current_slide += 1;
        }
    }

    pub fn go_prev(&mut self) {
        if self.loaded && self.current_slide > 1 {
            self.current_slide -= 1;
        }
    }

    pub fn go_first(&mut self) {
        if self.loaded && self.total_slides > 0 {
            self.current_slide = 1;
        }
    }

    pub fn go_last(&mut self) {
        if self.loaded {
            self.current_slide = self.total_slides;
        }
    }

    pub fn go_to(&mut self, slide: u32) {
        if self.loaded && self.total_slides > 0 {
            self.current_slide = slide.max(1).min(self.total_slides);
        }
    }

    /// Replace the paragraphs of a slide from plain editor lines.
    ///
    /// Each text string becomes a single-line paragraph, preserving the
    /// original alignment and font size where possible.
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
                    let old_para = old.get(i);
                    let align = old_para
                        .map(|p| p.align.as_str())
                        .unwrap_or("left")
                        .to_string();
                    let font_size_pt = old_para.map(|p| p.font_size_pt).unwrap_or(28.0);
                    ParagraphContent {
                        lines: vec![text],
                        align,
                        font_size_pt,
                    }
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
