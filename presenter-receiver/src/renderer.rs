use cairo::{Context, Format, ImageSurface};
use pango::{Alignment, FontDescription, WrapMode};
use pangocairo::functions as pc;

const FONT: &str = "Helvetica Neue";
const BG: (f64, f64, f64) = (0.05, 0.05, 0.08);
const FG: (f64, f64, f64) = (1.0, 1.0, 1.0);
const ACCENT: (f64, f64, f64) = (0.3, 0.3, 0.8);

/// Render slide paragraphs into a pixel buffer sized to the given display dimensions.
///
/// Each entry in `paragraphs` is `(text, align)` where `align` is one of
/// `"left"`, `"center"`, `"right"`, or `"justify"` — matching the server's
/// `ParagraphContent.align` field.
///
/// All proportions (padding, accent line, font size) scale with `width`/`height`
/// so the result looks identical whether the display is 720p, 1080p, or 4K.
///
/// Returns raw RGB24 bytes: `width × height × 3` bytes, row-major.
pub fn render_slide(paragraphs: &[(&str, &str)], width: u32, height: u32) -> Vec<u8> {
    let (sw, sh) = (width as i32, height as i32);
    let mut surface = ImageSurface::create(Format::Rgb24, sw, sh).unwrap();
    let ctx = Context::new(&surface).unwrap();

    let w = width as f64;
    let h = height as f64;
    let pad_x = w * 0.10;
    let text_w = ((w - pad_x * 2.0) * pango::SCALE as f64) as i32;

    // ── Background ───────────────────────────────────────────────────────────
    ctx.set_source_rgb(BG.0, BG.1, BG.2);
    ctx.paint().unwrap();

    // ── Top accent line — thickness scales with height ───────────────────────
    ctx.set_source_rgb(ACCENT.0, ACCENT.1, ACCENT.2);
    ctx.set_line_width((h * 0.006).max(2.0));
    ctx.move_to(pad_x, h * 0.06);
    ctx.line_to(w - pad_x, h * 0.06);
    ctx.stroke().unwrap();

    // ── Text — binary search for the largest bold size that fits ─────────────
    let non_empty: Vec<(&str, &str)> = paragraphs
        .iter()
        .copied()
        .filter(|(t, _)| !t.is_empty())
        .collect();

    if !non_empty.is_empty() {
        // Safe area: 10 % top (accent line) + 10 % bottom margin
        let max_h = (h * 0.80) as i32;

        let make_layouts = |font_size: i32| -> Vec<pango::Layout> {
            non_empty
                .iter()
                .map(|(text, align)| {
                    let layout = pc::create_layout(&ctx);
                    layout.set_font_description(Some(&FontDescription::from_string(
                        &format!("{FONT} Bold {font_size}"),
                    )));
                    layout.set_width(text_w);
                    layout.set_alignment(parse_alignment(align));
                    layout.set_wrap(WrapMode::Word);
                    layout.set_spacing(
                        (font_size as f64 * 0.20 * pango::SCALE as f64) as i32,
                    );
                    layout.set_text(text);
                    layout
                })
                .collect()
        };

        // Total height = sum of all layout heights + paragraph gaps between them
        let total_height = |layouts: &[pango::Layout], font_size: i32| -> i32 {
            let gap = (font_size as f64 * 0.50) as i32;
            let text_h: i32 = layouts.iter().map(|l| l.pixel_size().1).sum();
            let gaps = gap * (layouts.len().saturating_sub(1) as i32);
            text_h + gaps
        };

        let mut lo = 8i32;
        let mut hi = (h * 0.40) as i32;

        while lo < hi - 1 {
            let mid = (lo + hi) / 2;
            if total_height(&make_layouts(mid), mid) <= max_h {
                lo = mid;
            } else {
                hi = mid;
            }
        }

        let layouts = make_layouts(lo);
        let gap = (lo as f64 * 0.50) as i32;
        let used_h = total_height(&layouts, lo);

        // Vertically centre the block in the safe area (below accent line)
        let safe_top = h * 0.10;
        let safe_h = h * 0.80;
        let mut y = safe_top + (safe_h - used_h as f64) / 2.0;

        ctx.set_source_rgb(FG.0, FG.1, FG.2);
        for layout in &layouts {
            ctx.move_to(pad_x, y);
            pc::show_layout(&ctx, layout);
            y += layout.pixel_size().1 as f64 + gap as f64;
        }
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
