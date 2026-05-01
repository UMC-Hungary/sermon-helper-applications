use cairo::{Context, Format, ImageSurface};
use pango::{Alignment, FontDescription};
use pangocairo::functions as pc;
use std::f64::consts::PI;

const FONT: &str = "Helvetica Neue";
const BG: (f64, f64, f64) = (0.05, 0.05, 0.08);
const FG: (f64, f64, f64) = (1.0, 1.0, 1.0);
const ACCENT: (f64, f64, f64) = (0.3, 0.3, 0.8);

/// Render slide paragraphs into a pixel buffer sized to the given display dimensions.
///
/// Each entry in `paragraphs` is `(text, align, font_size_pt)` where `align` is one of
/// `"left"`, `"center"`, `"right"`, or `"justify"` — matching the server's
/// `ParagraphContent.align` field, and `font_size_pt` is the font size from the PPTX
/// (0 when unknown / legacy format).
///
/// If the last paragraph is center-aligned and its `font_size_pt` is less than 70% of
/// the largest size in the slide, it is treated as a slide counter and rendered at the
/// bottom of the safe area instead of being part of the main vertically-centred block.
///
/// All proportions (padding, accent line, font size) scale with `width`/`height`
/// so the result looks identical whether the display is 720p, 1080p, or 4K.
///
/// Returns raw RGB24 bytes: `width × height × 3` bytes, row-major.
pub fn render_slide(paragraphs: &[(&str, &str, f64)], width: u32, height: u32) -> Vec<u8> {
    let (sw, sh) = (width as i32, height as i32);
    let mut surface = ImageSurface::create(Format::Rgb24, sw, sh).unwrap();
    let ctx = Context::new(&surface).unwrap();

    let w = width as f64;
    let h = height as f64;
    let pad_x = w * 0.10;

    // ── Background ───────────────────────────────────────────────────────────
    ctx.set_source_rgb(BG.0, BG.1, BG.2);
    ctx.paint().unwrap();

    // ── Top accent line — thickness scales with height ───────────────────────
    ctx.set_source_rgb(ACCENT.0, ACCENT.1, ACCENT.2);
    ctx.set_line_width((h * 0.006).max(2.0));
    ctx.move_to(pad_x, h * 0.06);
    ctx.line_to(w - pad_x, h * 0.06);
    ctx.stroke().unwrap();

    // ── Detect counter paragraph ─────────────────────────────────────────────
    // The counter (slide number / verse ref) is center-aligned and its
    // font_size_pt is < 85 % of the max on the slide.  In some PPTXes the
    // counter text box appears first in the XML, in others it is last — so we
    // check both ends.
    let non_empty: Vec<(&str, &str, f64)> = paragraphs
        .iter()
        .copied()
        .filter(|(t, _, _)| !t.is_empty())
        .collect();

    let max_pt = non_empty.iter().map(|(_, _, pt)| *pt).fold(0.0f64, f64::max);
    let is_counter = |p: (&str, &str, f64)| -> bool {
        max_pt > 0.0 && p.2 > 0.0 && p.2 < max_pt * 0.85 && p.1 == "center"
    };
    let (main_paras, counter_para): (Vec<(&str, &str, f64)>, Option<(&str, &str, f64)>) =
        if non_empty.len() >= 2 {
            let first = *non_empty.first().unwrap();
            let last = *non_empty.last().unwrap();
            if is_counter(first) {
                (non_empty[1..].to_vec(), Some(first))
            } else if is_counter(last) {
                (non_empty[..non_empty.len() - 1].to_vec(), Some(last))
            } else {
                (non_empty.clone(), None)
            }
        } else {
            (non_empty.clone(), None)
        };

    // ── Text — binary search for the largest bold size that fits ─────────────
    let max_w = (w - pad_x * 2.0) as i32;
    // Reserve space at the bottom when a counter is present.
    let main_max_h = if counter_para.is_some() {
        (h * 0.68) as i32
    } else {
        (h * 0.80) as i32
    };

    // `lo` is updated inside the block below; used by counter sizing after.
    let mut found_main_size = 8i32;

    if !main_paras.is_empty() {
        let make_layouts = |font_size: i32| -> Vec<pango::Layout> {
            main_paras
                .iter()
                .map(|(text, align, _)| {
                    let layout = pc::create_layout(&ctx);
                    layout.set_font_description(Some(&FontDescription::from_string(
                        &format!("{FONT} Bold {font_size}"),
                    )));
                    layout.set_alignment(parse_alignment(align));
                    layout.set_text(text);
                    layout
                })
                .collect()
        };

        let fits = |layouts: &[pango::Layout], font_size: i32| -> bool {
            let gap = (font_size as f64 * 0.50) as i32;
            let text_h: i32 = layouts.iter().map(|l| l.pixel_size().1).sum();
            let gaps = gap * (layouts.len().saturating_sub(1) as i32);
            let max_line_w = layouts.iter().map(|l| l.pixel_size().0).max().unwrap_or(0);
            text_h + gaps <= main_max_h && max_line_w <= max_w
        };

        let mut lo = 8i32;
        let mut hi = (h * 0.40) as i32;

        while lo < hi - 1 {
            let mid = (lo + hi) / 2;
            if fits(&make_layouts(mid), mid) {
                lo = mid;
            } else {
                hi = mid;
            }
        }

        found_main_size = lo;
        let layouts = make_layouts(lo);
        let gap = (lo as f64 * 0.50) as i32;
        let text_h: i32 = layouts.iter().map(|l| l.pixel_size().1).sum();
        let used_h = text_h + gap * (layouts.len().saturating_sub(1) as i32);

        // Vertically centre the main block in its safe area.
        let safe_top = h * 0.10;
        let safe_h = main_max_h as f64;
        let mut y = safe_top + (safe_h - used_h as f64) / 2.0;

        ctx.set_source_rgb(FG.0, FG.1, FG.2);
        for layout in &layouts {
            ctx.move_to(pad_x, y);
            pc::show_layout(&ctx, layout);
            y += layout.pixel_size().1 as f64 + gap as f64;
        }
    }

    // ── Counter paragraph at the bottom ──────────────────────────────────────
    if let Some((text, _align, counter_pt)) = counter_para {
        let counter_size = if max_pt > 0.0 && counter_pt > 0.0 {
            ((found_main_size as f64) * (counter_pt / max_pt)).max(8.0) as i32
        } else {
            (found_main_size as f64 * 0.50).max(8.0) as i32
        };
        let layout = pc::create_layout(&ctx);
        layout.set_font_description(Some(&FontDescription::from_string(
            &format!("{FONT} Bold {counter_size}"),
        )));
        layout.set_text(text);
        let (lw, lh) = layout.pixel_size();
        // Horizontally centre the counter within the text area.
        let counter_x = pad_x + (max_w as f64 - lw as f64) / 2.0;
        // Bottom of safe area minus a small margin.
        let counter_y = h * 0.90 - lh as f64;
        ctx.set_source_rgb(FG.0, FG.1, FG.2);
        ctx.move_to(counter_x, counter_y);
        pc::show_layout(&ctx, &layout);
    }

    // ── Extract RGB24 bytes ──────────────────────────────────────────────────
    drop(ctx); // must be dropped before surface.data()
    surface.flush();

    let stride = surface.stride() as usize;
    let data = surface.data().unwrap();
    let mut out = Vec::with_capacity((sw * sh * 3) as usize);
    for row in 0..sh as usize {
        for col in 0..sw as usize {
            let i = row * stride + col * 4;
            // Cairo RGB24: 0x00RRGGBB in native byte order → [B, G, R, X]
            out.push(data[i + 2]); // R
            out.push(data[i + 1]); // G
            out.push(data[i]);     // B
        }
    }
    out
}

/// Composite a small status indicator (coloured dot + optional version text)
/// onto the top-right corner of `frame` using alpha blending.
///
/// - Green  = Connected (also shows the binary version)
/// - Orange = Connecting / Reconnecting
/// - Red    = Failed (5+ consecutive connection errors)
pub fn draw_status_overlay(
    frame: &mut Vec<u32>,
    width: u32,
    height: u32,
    state: crate::ConnectionState,
) {
    let h = height as f64;

    let dot_r = (h * 0.013).max(9.0);
    let padding = dot_r * 0.9;
    let font_size = ((dot_r * 1.05) as i32).max(10);

    let (dr, dg, db) = match state {
        crate::ConnectionState::Connected => (0.0f64, 0.78, 0.38),
        crate::ConnectionState::Connecting | crate::ConnectionState::Reconnecting => (1.0, 0.55, 0.0),
        crate::ConnectionState::Failed => (1.0, 0.22, 0.22),
    };

    // Version label shown only when connected.
    let version_text: Option<String> = if matches!(state, crate::ConnectionState::Connected) {
        Some(format!("v{}", env!("CARGO_PKG_VERSION")))
    } else {
        None
    };

    // Measure version text width using a throw-away surface.
    let text_w = if let Some(ref text) = version_text {
        let tmp = ImageSurface::create(Format::ARgb32, 1, 1).unwrap();
        let tmp_ctx = Context::new(&tmp).unwrap();
        let layout = pc::create_layout(&tmp_ctx);
        layout.set_font_description(Some(&FontDescription::from_string(
            &format!("{FONT} {font_size}"),
        )));
        layout.set_text(text);
        layout.pixel_size().0 as f64
    } else {
        0.0
    };

    let gap = dot_r * 0.65;
    let content_w = dot_r * 2.0 + if text_w > 0.0 { gap + text_w } else { 0.0 };
    let pill_w = content_w + padding * 2.0;
    let pill_h = dot_r * 2.0 + padding * 2.0;
    let ow = (pill_w + padding).ceil() as i32 + 2;
    let oh = (pill_h + padding).ceil() as i32 + 2;

    let mut surface = ImageSurface::create(Format::ARgb32, ow, oh).unwrap();
    let ctx = Context::new(&surface).unwrap();

    // Transparent background.
    ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    ctx.paint().unwrap();

    // Semi-transparent pill behind the indicator.
    let px = padding / 2.0;
    let py = padding / 2.0;
    let radius = pill_h / 2.0;
    ctx.set_source_rgba(0.0, 0.0, 0.0, 0.55);
    ctx.new_path();
    ctx.arc(px + radius,          py + radius,          radius, PI,          3.0 * PI / 2.0);
    ctx.arc(px + pill_w - radius, py + radius,          radius, -PI / 2.0,  0.0);
    ctx.arc(px + pill_w - radius, py + pill_h - radius, radius, 0.0,         PI / 2.0);
    ctx.arc(px + radius,          py + pill_h - radius, radius, PI / 2.0,   PI);
    ctx.close_path();
    ctx.fill().unwrap();

    // Dot sits on the right side of the pill.
    let dot_cx = px + pill_w - padding - dot_r;
    let dot_cy = py + pill_h / 2.0;

    // Version text to the left of the dot.
    if let Some(ref text) = version_text {
        let layout = pc::create_layout(&ctx);
        layout.set_font_description(Some(&FontDescription::from_string(
            &format!("{FONT} {font_size}"),
        )));
        layout.set_text(text);
        let text_h = layout.pixel_size().1 as f64;
        ctx.set_source_rgba(1.0, 1.0, 1.0, 0.9);
        ctx.move_to(dot_cx - dot_r - gap - text_w, dot_cy - text_h / 2.0);
        pc::show_layout(&ctx, &layout);
    }

    // Filled dot.
    ctx.set_source_rgb(dr, dg, db);
    ctx.arc(dot_cx, dot_cy, dot_r, 0.0, 2.0 * PI);
    ctx.fill().unwrap();

    drop(ctx);
    surface.flush();

    // Alpha-composite overlay onto the frame (top-right corner).
    let stride = surface.stride() as usize;
    let data = surface.data().unwrap();
    let frame_w = width as usize;
    let frame_h = height as usize;
    let start_x = (frame_w as i32 - ow).max(0) as usize;

    for row in 0..oh as usize {
        for col in 0..ow as usize {
            let fx = start_x + col;
            let fy = row;
            if fx >= frame_w || fy >= frame_h {
                continue;
            }
            let i = row * stride + col * 4;
            // Cairo ARgb32 (little-endian): B G R A
            let a = data[i + 3] as f64 / 255.0;
            if a < 0.01 {
                continue;
            }
            let sr = data[i + 2] as f64;
            let sg = data[i + 1] as f64;
            let sb = data[i] as f64;
            let dst = frame[fy * frame_w + fx];
            let dr_dst = ((dst >> 16) & 0xff) as f64;
            let dg_dst = ((dst >> 8) & 0xff) as f64;
            let db_dst = (dst & 0xff) as f64;
            let inv = 1.0 - a;
            let r = (sr * a + dr_dst * inv) as u32;
            let g = (sg * a + dg_dst * inv) as u32;
            let b = (sb * a + db_dst * inv) as u32;
            frame[fy * frame_w + fx] = (r << 16) | (g << 8) | b;
        }
    }
}

fn parse_alignment(align: &str) -> Alignment {
    match align {
        "center" => Alignment::Center,
        "right" => Alignment::Right,
        _ => Alignment::Left, // "left", "justify", or anything unknown
    }
}

/// Convert RGB24 bytes → 0x00RRGGBB u32 per pixel (minifb format).
pub fn rgb_to_u32(rgb: &[u8]) -> Vec<u32> {
    rgb.chunks_exact(3)
        .map(|p| ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | p[2] as u32)
        .collect()
}
