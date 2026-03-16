fn main() {
    // Expose TARGET triple so mediamtx.rs can locate the dev binary at runtime.
    if let Ok(target) = std::env::var("TARGET") {
        println!("cargo:rustc-env=TARGET={target}");
    }

    // Decode the base64-encoded shader content bundled in overlay.dat.
    use base64::Engine as _;
    let b64 = std::fs::read_to_string("overlay.dat").unwrap_or_default();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64.trim())
        .unwrap_or_default();
    let shader_content = String::from_utf8(bytes).unwrap_or_default();

    // Escape backslashes and double-quotes so the content is valid inside a Rust string literal.
    let escaped = shader_content.replace('\\', "\\\\").replace('"', "\\\"");
    let shader_rs = format!("pub const SHADER_CONTENT: &str = \"{escaped}\";");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    std::fs::write(format!("{out_dir}/shader_content.rs"), shader_rs)
        .expect("Failed to write shader_content.rs");

    println!("cargo:rerun-if-changed=overlay.dat");

    tauri_build::build()
}
