fn main() {
    // Set build date as environment variable for compile-time access
    let build_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // Set Rust version
    let rust_version = rustc_version::version()
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=RUST_VERSION={}", rust_version);

    tauri_build::build()
}
