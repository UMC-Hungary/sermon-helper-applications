fn main() {
    // Expose TARGET triple so mediamtx.rs can locate the dev binary at runtime.
    if let Ok(target) = std::env::var("TARGET") {
        println!("cargo:rustc-env=TARGET={target}");
    }

    // Embed the LucidGlass shader content.
    // The file is NOT committed to the repository.
    // Priority:
    //   1. LUCID_GLASS_SHADER_CONTENT env var (set this in CI as a build secret)
    //   2. LucidGlass.shader file in src-tauri/ (place it locally for dev builds)
    //   3. Empty string — badge install will succeed but the shader file will be empty
    let shader_content = std::env::var("LUCID_GLASS_SHADER_CONTENT")
        .unwrap_or_else(|_| std::fs::read_to_string("LucidGlass.shader").unwrap_or_default());

    // Escape backslashes and double-quotes so the content is valid inside a Rust string literal.
    let escaped = shader_content.replace('\\', "\\\\").replace('"', "\\\"");
    let shader_rs = format!("pub const SHADER_CONTENT: &str = \"{escaped}\";");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    std::fs::write(format!("{out_dir}/shader_content.rs"), shader_rs)
        .expect("Failed to write shader_content.rs");

    println!("cargo:rerun-if-env-changed=LUCID_GLASS_SHADER_CONTENT");
    println!("cargo:rerun-if-changed=LucidGlass.shader");

    tauri_build::build()
}
