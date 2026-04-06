mod renderer;
mod ws;

use ws::{DisplayDims, Frame};

// ── CLI ───────────────────────────────────────────────────────────────────────

fn usage() -> ! {
    eprintln!(concat!(
        "Usage: presenter-receiver <ws-url> [--token <token>]\n",
        "\n",
        "Examples:\n",
        "  presenter-receiver ws://192.168.1.10:3000/ws\n",
        "  presenter-receiver ws://192.168.1.10:3000/ws --token abc123",
    ));
    std::process::exit(1);
}

fn parse_args() -> String {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let host = args.first().unwrap_or_else(|| usage()).clone();
    if host == "--help" || host == "-h" {
        usage();
    }

    let token = args
        .windows(2)
        .find(|w| w[0] == "--token")
        .map(|w| w[1].clone());

    match token {
        Some(t) => format!("{host}?token={t}"),
        None => host,
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let url = parse_args();
    let dims = detect_display_dims();

    eprintln!(
        "[display] {}×{} — connecting to {url}",
        dims.width, dims.height
    );

    // Channel: WS background thread → display main thread
    let (tx, rx) = std::sync::mpsc::channel::<Frame>();

    // Spawn tokio runtime + WS loop on a background thread.
    // (On macOS, minifb requires the display loop to own the main thread.)
    {
        let url = url.clone();
        std::thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(ws::run(url, tx, dims));
        });
    }

    run_display(rx, dims);
}

// ── Display dimensions ────────────────────────────────────────────────────────

fn detect_display_dims() -> DisplayDims {
    #[cfg(target_os = "linux")]
    {
        // Read actual framebuffer resolution so the renderer fills the screen exactly
        if let Ok(fb) = framebuffer::Framebuffer::new("/dev/fb0") {
            return DisplayDims {
                width: fb.var_screen_info.xres,
                height: fb.var_screen_info.yres,
            };
        }
    }

    // macOS default window size (or Linux fallback when /dev/fb0 is unavailable)
    DisplayDims {
        width: 1280,
        height: 720,
    }
}

// ── macOS display: minifb window ──────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn run_display(rx: std::sync::mpsc::Receiver<Frame>, dims: DisplayDims) {
    use minifb::{Key, Window, WindowOptions};

    let w = dims.width as usize;
    let h = dims.height as usize;

    let mut window = Window::new(
        "Presenter Receiver",
        w,
        h,
        WindowOptions {
            resize: false,
            ..Default::default()
        },
    )
    .expect("Failed to open window");

    window.set_target_fps(30);

    // Black frame shown while waiting for the first slide
    let mut current: Frame = vec![0u32; w * h];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Pick up the latest frame if one arrived; skip stale ones
        while let Ok(frame) = rx.try_recv() {
            current = frame;
        }
        window.update_with_buffer(&current, w, h).unwrap();
    }
}

// ── Linux display: framebuffer (/dev/fb0) ─────────────────────────────────────

#[cfg(target_os = "linux")]
fn run_display(rx: std::sync::mpsc::Receiver<Frame>, dims: DisplayDims) {
    let mut fb = framebuffer::Framebuffer::new("/dev/fb0").expect("Cannot open /dev/fb0");
    let fb_w = fb.var_screen_info.xres as usize;
    let fb_h = fb.var_screen_info.yres as usize;
    let bpp = fb.var_screen_info.bits_per_pixel as usize / 8;
    let src_w = dims.width as usize;
    let src_h = dims.height as usize;

    // Block on each incoming frame — no idle loop needed
    for frame in rx {
        let mut fb_frame = vec![0u8; fb_w * fb_h * bpp];
        for y in 0..src_h.min(fb_h) {
            for x in 0..src_w.min(fb_w) {
                let pixel = frame[y * src_w + x];
                let r = ((pixel >> 16) & 0xff) as u8;
                let g = ((pixel >> 8) & 0xff) as u8;
                let b = (pixel & 0xff) as u8;
                let d = (y * fb_w + x) * bpp;
                // Framebuffer colour order is BGR(A)
                fb_frame[d] = b;
                fb_frame[d + 1] = g;
                fb_frame[d + 2] = r;
                if bpp == 4 {
                    fb_frame[d + 3] = 0xff;
                }
            }
        }
        fb.write_frame(&fb_frame);
    }
}
