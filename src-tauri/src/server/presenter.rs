use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Read;
use std::path::Path;

use crate::server::AppState;

#[path = "presenter_svg.rs"]
mod presenter_svg;

const MAX_BIBLE_WORDS_PER_SLIDE: usize = 55;
const BIBLE_MAIN_FONT_SIZE_PT: f32 = 38.0;
const BIBLE_COUNTER_FONT_SIZE_PT: f32 = 18.0;

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
pub struct SvgSlideContent {
    pub index: u32,
    pub svg: String,
    pub width_px: u32,
    pub height_px: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PresenterRenderMode {
    Text,
    Svg,
}

impl Default for PresenterRenderMode {
    fn default() -> Self {
        Self::Text
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BibleReferenceType {
    Textus,
    Leckio,
}

impl BibleReferenceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Textus => "textus",
            Self::Leckio => "leckio",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Textus => "Textus",
            Self::Leckio => "Lekció",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibleVerseContent {
    pub chapter: u32,
    pub verse: u32,
    pub text: String,
}

#[derive(Debug, Clone)]
struct BibleVerseChunk {
    chapter: u32,
    verse: u32,
    text: String,
}

#[derive(Debug, Clone)]
struct BibleSlidePage {
    chunks: Vec<BibleVerseChunk>,
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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedSvgPresentation {
    pub file_path: String,
    pub total_slides: u32,
    pub slides: Vec<SlideContent>,
    pub svg_slides: Vec<SvgSlideContent>,
    pub slide_width_emu: u64,
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
    let file = std::fs::File::open(file_path).map_err(|e| format!("Cannot open file: {e}"))?;

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

/// Parse a `.pptx` file into both the legacy text model and self-contained SVG slides.
pub fn parse_pptx_svg(file_path: &str) -> Result<ParsedSvgPresentation, String> {
    let parsed_text = parse_pptx(file_path)?;
    let deck = presenter_svg::convert_pptx_to_inline_svg(Path::new(file_path))
        .map_err(|e| format!("Cannot convert presentation to SVG: {e}"))?;

    Ok(ParsedSvgPresentation {
        file_path: file_path.to_string(),
        total_slides: deck.slides.len() as u32,
        slides: parsed_text.slides,
        svg_slides: deck
            .slides
            .into_iter()
            .map(|slide| SvgSlideContent {
                index: slide.index,
                svg: slide.svg,
                width_px: slide.width_px,
                height_px: slide.height_px,
            })
            .collect(),
        slide_width_emu: deck.width_emu,
        slide_height_emu: deck.height_emu,
    })
}

pub fn load_pptx(
    file_path: &str,
    render_mode: PresenterRenderMode,
) -> Result<PresenterState, String> {
    match render_mode {
        PresenterRenderMode::Text => parse_pptx(file_path).map(PresenterState::from_parsed),
        PresenterRenderMode::Svg => parse_pptx_svg(file_path).map(PresenterState::from_svg),
    }
}

// ── Live presenter state ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenterState {
    pub loaded: bool,
    pub file_path: Option<String>,
    pub current_slide: u32,
    pub total_slides: u32,
    pub render_mode: PresenterRenderMode,
    pub slides: Vec<SlideContent>,
    pub svg_slides: Vec<SvgSlideContent>,
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
            render_mode: PresenterRenderMode::Text,
            slides: Vec::new(),
            svg_slides: Vec::new(),
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
            render_mode: PresenterRenderMode::Text,
            slides: parsed.slides,
            svg_slides: Vec::new(),
            muted: false,
            slide_width_emu: parsed.slide_width_emu,
            slide_height_emu: parsed.slide_height_emu,
        }
    }

    pub fn from_svg(parsed: ParsedSvgPresentation) -> Self {
        let total = parsed.total_slides;
        Self {
            loaded: true,
            file_path: Some(parsed.file_path),
            current_slide: if total > 0 { 1 } else { 0 },
            total_slides: total,
            render_mode: PresenterRenderMode::Svg,
            slides: parsed.slides,
            svg_slides: parsed.svg_slides,
            muted: false,
            slide_width_emu: parsed.slide_width_emu,
            slide_height_emu: parsed.slide_height_emu,
        }
    }

    pub fn from_bible_reference(
        event_title: &str,
        reference_type: BibleReferenceType,
        reference: &str,
        verses: Vec<BibleVerseContent>,
    ) -> Self {
        let pages = paginate_bible_verses(verses, MAX_BIBLE_WORDS_PER_SLIDE);
        let total = pages.len() as u32;
        let slides = pages
            .iter()
            .enumerate()
            .map(|(idx, page)| {
                let slide_number = idx as u32 + 1;
                let verse_range = format_bible_page_range(page);
                let counter = format_bible_counter(
                    reference_type.display_name(),
                    reference,
                    &verse_range,
                    slide_number,
                    total,
                );
                let mut paragraphs = vec![ParagraphContent {
                    lines: vec![format_bible_page_text(page)],
                    align: "left".to_string(),
                    font_size_pt: BIBLE_MAIN_FONT_SIZE_PT,
                }];
                paragraphs.push(ParagraphContent {
                    lines: vec![counter],
                    align: "center".to_string(),
                    font_size_pt: BIBLE_COUNTER_FONT_SIZE_PT,
                });
                SlideContent {
                    index: slide_number,
                    paragraphs,
                }
            })
            .collect();

        Self {
            loaded: total > 0,
            file_path: Some(format!(
                "{} - {} {}",
                event_title,
                reference_type.display_name(),
                reference
            )),
            current_slide: if total > 0 { 1 } else { 0 },
            total_slides: total,
            render_mode: PresenterRenderMode::Text,
            slides,
            svg_slides: Vec::new(),
            muted: false,
            slide_width_emu: 12192000,
            slide_height_emu: 6858000,
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
        if !self.loaded || self.render_mode != PresenterRenderMode::Text {
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

fn paginate_bible_verses(
    verses: Vec<BibleVerseContent>,
    max_words_per_slide: usize,
) -> Vec<BibleSlidePage> {
    let max_words_per_slide = max_words_per_slide.max(1);
    let mut pages = Vec::new();
    let mut current_chunks = Vec::new();
    let mut current_word_count = 0usize;

    for verse in verses {
        let words = verse.text.split_whitespace().collect::<Vec<_>>();
        let word_count = words.len().max(1);

        if word_count <= max_words_per_slide {
            if current_word_count > 0 && current_word_count + word_count > max_words_per_slide {
                push_bible_page(&mut pages, &mut current_chunks, &mut current_word_count);
            }
            current_chunks.push(BibleVerseChunk {
                chapter: verse.chapter,
                verse: verse.verse,
                text: verse.text.trim().to_string(),
            });
            current_word_count += word_count;
            continue;
        }

        push_bible_page(&mut pages, &mut current_chunks, &mut current_word_count);

        for chunk_words in words.chunks(max_words_per_slide) {
            current_chunks.push(BibleVerseChunk {
                chapter: verse.chapter,
                verse: verse.verse,
                text: chunk_words.join(" "),
            });
            current_word_count = chunk_words.len();
            push_bible_page(&mut pages, &mut current_chunks, &mut current_word_count);
        }
    }

    push_bible_page(&mut pages, &mut current_chunks, &mut current_word_count);
    pages
}

fn push_bible_page(
    pages: &mut Vec<BibleSlidePage>,
    chunks: &mut Vec<BibleVerseChunk>,
    word_count: &mut usize,
) {
    if !chunks.is_empty() {
        pages.push(BibleSlidePage {
            chunks: std::mem::take(chunks),
        });
        *word_count = 0;
    }
}

fn format_bible_page_range(page: &BibleSlidePage) -> String {
    let Some(first) = page.chunks.first() else {
        return String::new();
    };
    let last = page.chunks.last().unwrap_or(first);
    if first.chapter == last.chapter && first.verse == last.verse {
        format!("{}:{}", first.chapter, first.verse)
    } else if first.chapter == last.chapter {
        format!("{}:{}-{}", first.chapter, first.verse, last.verse)
    } else {
        format!(
            "{}:{}-{}:{}",
            first.chapter, first.verse, last.chapter, last.verse
        )
    }
}

fn format_bible_page_text(page: &BibleSlidePage) -> String {
    page.chunks
        .iter()
        .map(|chunk| chunk.text.trim())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_bible_counter(
    display_name: &str,
    reference: &str,
    verse_range: &str,
    slide_number: u32,
    total: u32,
) -> String {
    let trimmed_reference = reference.trim();
    if trimmed_reference.is_empty() {
        format!("{display_name} {verse_range} ({slide_number}/{total})")
    } else {
        format!("{display_name} {trimmed_reference} | {verse_range} ({slide_number}/{total})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verse(chapter: u32, verse: u32, text: &str) -> BibleVerseContent {
        BibleVerseContent {
            chapter,
            verse,
            text: text.to_string(),
        }
    }

    #[test]
    fn paginates_short_verses_without_splitting_them() {
        let pages = paginate_bible_verses(
            vec![
                verse(3, 16, "one two three"),
                verse(3, 17, "four five"),
                verse(3, 18, "six seven eight"),
            ],
            5,
        );

        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].chunks.len(), 2);
        assert_eq!(format_bible_page_range(&pages[0]), "3:16-17");
        assert_eq!(pages[1].chunks[0].verse, 18);
    }

    #[test]
    fn splits_single_oversized_verse_by_word_limit() {
        let pages = paginate_bible_verses(vec![verse(4, 1, "one two three four five six")], 2);

        assert_eq!(pages.len(), 3);
        assert_eq!(pages[0].chunks[0].text, "one two");
        assert_eq!(pages[1].chunks[0].text, "three four");
        assert_eq!(pages[2].chunks[0].text, "five six");
        assert_eq!(format_bible_page_range(&pages[2]), "4:1");
    }

    #[test]
    fn bible_reference_slide_joins_verses_into_one_plain_paragraph() {
        let state = PresenterState::from_bible_reference(
            "Sunday",
            BibleReferenceType::Textus,
            "Jn 3:16-17",
            vec![
                verse(3, 16, "For God so loved the world."),
                verse(3, 17, "God did not send the Son to condemn the world."),
            ],
        );

        let slide = state.slides.first().unwrap();
        assert_eq!(slide.paragraphs.len(), 2);
        assert_eq!(slide.paragraphs[0].lines.len(), 1);
        assert_eq!(
            slide.paragraphs[0].lines.join(" "),
            "For God so loved the world. God did not send the Son to condemn the world."
        );
        assert_eq!(slide.paragraphs[0].align, "left");
        assert!(slide.paragraphs[1].lines[0].contains("Jn 3:16-17"));
    }

    #[test]
    fn written_pptx_reparses_to_the_same_slides() {
        let state = PresenterState::from_bible_reference(
            "Sunday",
            BibleReferenceType::Textus,
            "Jn 3:16",
            vec![verse(3, 16, "For God so loved the world.")],
        );
        let path = std::env::temp_dir().join(format!("metocast-test-{}.pptx", std::process::id()));
        write_pptx(
            &path,
            &state.slides,
            state.slide_width_emu,
            state.slide_height_emu,
        )
        .unwrap();

        let parsed = parse_pptx(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(parsed.total_slides, state.total_slides);
        assert_eq!(parsed.slide_width_emu, state.slide_width_emu);
        let written = &parsed.slides[0].paragraphs;
        let expected = &state.slides[0].paragraphs;
        assert_eq!(written.len(), expected.len());
        assert_eq!(written[0].lines, expected[0].lines);
        assert_eq!(written[0].align, expected[0].align);
        assert_eq!(written[0].font_size_pt, expected[0].font_size_pt);
        assert_eq!(written[1].align, "center");
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

// ── PPTX writing ──────────────────────────────────────────────────────────────

const THEME_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Metocast"><a:themeElements><a:clrScheme name="Metocast"><a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1><a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="000000"/></a:dk2><a:lt2><a:srgbClr val="FFFFFF"/></a:lt2><a:accent1><a:srgbClr val="4472C4"/></a:accent1><a:accent2><a:srgbClr val="ED7D31"/></a:accent2><a:accent3><a:srgbClr val="A5A5A5"/></a:accent3><a:accent4><a:srgbClr val="FFC000"/></a:accent4><a:accent5><a:srgbClr val="5B9BD5"/></a:accent5><a:accent6><a:srgbClr val="70AD47"/></a:accent6><a:hlink><a:srgbClr val="0563C1"/></a:hlink><a:folHlink><a:srgbClr val="954F72"/></a:folHlink></a:clrScheme><a:fontScheme name="Metocast"><a:majorFont><a:latin typeface="Helvetica Neue"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Helvetica"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme><a:fmtScheme name="Metocast"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:fillStyleLst><a:lnStyleLst><a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln><a:ln w="12700"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln><a:ln w="19050"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:bgFillStyleLst></a:fmtScheme></a:themeElements></a:theme>"#;

const EMPTY_TREE: &str = r#"<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>"#;

const NS: &str = r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main""#;

const REL_NS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#;

/// `algn` attribute value for a CSS alignment keyword — the inverse of `map_align`.
fn pptx_align(css: &str) -> &'static str {
    match css {
        "center" => "ctr",
        "right" => "r",
        "justify" => "just",
        _ => "l",
    }
}

fn slide_xml(slide: &SlideContent, width_emu: u64, height_emu: u64) -> String {
    let margin_x = width_emu / 12;
    let margin_y = height_emu / 12;
    let paragraphs = slide
        .paragraphs
        .iter()
        .map(|p| {
            let sz = (p.font_size_pt * 100.0).round().max(100.0) as u32;
            let algn = pptx_align(&p.align);
            let runs = p
                .lines
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    format!(
                        r#"{}<a:r><a:rPr lang="hu-HU" sz="{sz}" b="1" dirty="0"><a:solidFill><a:srgbClr val="FFFFFF"/></a:solidFill></a:rPr><a:t>{}</a:t></a:r>"#,
                        if i > 0 { "<a:br/>" } else { "" },
                        quick_xml::escape::escape(line),
                    )
                })
                .collect::<String>();
            format!(r#"<a:p><a:pPr algn="{algn}"/>{runs}</a:p>"#)
        })
        .collect::<String>();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld {NS}><p:cSld><p:bg><p:bgPr><a:solidFill><a:srgbClr val="000000"/></a:solidFill><a:effectLst/></p:bgPr></p:bg><p:spTree>{EMPTY_TREE}<p:sp><p:nvSpPr><p:cNvPr id="2" name="Body"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="{margin_x}" y="{margin_y}"/><a:ext cx="{}" cy="{}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr><p:txBody><a:bodyPr wrap="square" anchor="ctr"><a:normAutofit/></a:bodyPr><a:lstStyle/>{paragraphs}</p:txBody></p:sp></p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>"#,
        width_emu - 2 * margin_x,
        height_emu - 2 * margin_y,
    )
}

/// Write `slides` as a minimal but valid `.pptx` package: one blank layout, one
/// master, white text on black, one full-bleed text box per slide.
pub fn write_pptx(
    path: &Path,
    slides: &[SlideContent],
    width_emu: u64,
    height_emu: u64,
) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| format!("Cannot write {path:?}: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default();
    let n = slides.len();

    let mut add = |name: &str, body: &str| -> Result<(), String> {
        zip.start_file(name, opts).map_err(|e| e.to_string())?;
        std::io::Write::write_all(&mut zip, body.as_bytes()).map_err(|e| e.to_string())
    };

    let overrides = (1..=n)
        .map(|i| format!(r#"<Override PartName="/ppt/slides/slide{i}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#))
        .collect::<String>();
    add(
        "[Content_Types].xml",
        &format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/><Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/><Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/><Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>{overrides}</Types>"#
        ),
    )?;

    add(
        "_rels/.rels",
        &format!(
            r#"{REL_NS}<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/></Relationships>"#
        ),
    )?;

    let sld_ids = (1..=n)
        .map(|i| format!(r#"<p:sldId id="{}" r:id="rId{}"/>"#, 255 + i, i + 1))
        .collect::<String>();
    add(
        "ppt/presentation.xml",
        &format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation {NS}><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst><p:sldIdLst>{sld_ids}</p:sldIdLst><p:sldSz cx="{width_emu}" cy="{height_emu}"/><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#
        ),
    )?;

    let slide_rels = (1..=n)
        .map(|i| format!(r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{i}.xml"/>"#, i + 1))
        .collect::<String>();
    add(
        "ppt/_rels/presentation.xml.rels",
        &format!(
            r#"{REL_NS}<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>{slide_rels}<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/></Relationships>"#,
            n + 2
        ),
    )?;

    add(
        "ppt/slideMasters/slideMaster1.xml",
        &format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster {NS}><p:cSld><p:spTree>{EMPTY_TREE}</p:spTree></p:cSld><p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/><p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst></p:sldMaster>"#
        ),
    )?;
    add(
        "ppt/slideMasters/_rels/slideMaster1.xml.rels",
        &format!(
            r#"{REL_NS}<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/></Relationships>"#
        ),
    )?;

    add(
        "ppt/slideLayouts/slideLayout1.xml",
        &format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout {NS} type="blank" preserve="1"><p:cSld name="Blank"><p:spTree>{EMPTY_TREE}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sldLayout>"#
        ),
    )?;
    add(
        "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
        &format!(
            r#"{REL_NS}<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/></Relationships>"#
        ),
    )?;

    add("ppt/theme/theme1.xml", THEME_XML)?;

    for (i, slide) in slides.iter().enumerate() {
        add(
            &format!("ppt/slides/slide{}.xml", i + 1),
            &slide_xml(slide, width_emu, height_emu),
        )?;
        add(
            &format!("ppt/slides/_rels/slide{}.xml.rels", i + 1),
            &format!(
                r#"{REL_NS}<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/></Relationships>"#
            ),
        )?;
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}
