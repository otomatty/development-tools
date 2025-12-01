fn main() {
    // Set build date as environment variable for compile-time access (UTC for consistency)
    let build_date = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // Set Leptos version from Cargo.toml
    // Note: We read the root Cargo.toml to get the leptos version
    let leptos_version = std::fs::read_to_string("../Cargo.toml")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|line| line.contains("leptos") && line.contains("version"))
                .and_then(|line| {
                    // Extract version from: leptos = { version = "0.7", ... }
                    line.split('"').nth(1).map(|v| v.to_string())
                })
        })
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=LEPTOS_VERSION={}", leptos_version);

    // Set Rust version
    let rust_version = rustc_version::version()
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=RUST_VERSION={}", rust_version);

    tauri_build::build()
}
