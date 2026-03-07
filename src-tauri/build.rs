fn main() {
    // Expose TARGET triple so mediamtx.rs can locate the dev binary at runtime.
    if let Ok(target) = std::env::var("TARGET") {
        println!("cargo:rustc-env=TARGET={target}");
    }
    tauri_build::build()
}
