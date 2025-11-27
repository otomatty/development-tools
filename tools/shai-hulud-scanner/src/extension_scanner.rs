//! VSCode and Cursor extension scanning functionality

use anyhow::Result;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

use crate::parsers::parse_package_json;
use crate::types::{FoundPackage, PackageSource};

/// Scan VSCode and Cursor extensions
pub fn scan_ide_extensions() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();

    // Scan VSCode extensions
    if let Ok(vscode_packages) = scan_vscode_extensions() {
        packages.extend(vscode_packages);
    }

    // Scan Cursor extensions
    if let Ok(cursor_packages) = scan_cursor_extensions() {
        packages.extend(cursor_packages);
    }

    Ok(packages)
}

/// Scan VSCode extensions
fn scan_vscode_extensions() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();
    let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // macOS: ~/Library/Application Support/Code/User/extensions/
    let macos_path = home
        .join("Library")
        .join("Application Support")
        .join("Code")
        .join("User")
        .join("extensions");

    // Linux: ~/.vscode/extensions/
    let linux_path = home.join(".vscode").join("extensions");

    // Windows: %APPDATA%\Code\User\extensions\
    let windows_path = dirs_next::data_dir()
        .map(|d| d.join("Code").join("User").join("extensions"))
        .unwrap_or_else(|| PathBuf::from(""));

    for ext_dir in &[macos_path, linux_path, windows_path] {
        if ext_dir.exists() {
            packages.extend(scan_extension_directory(ext_dir, PackageSource::VSCodeExtension)?);
        }
    }

    Ok(packages)
}

/// Scan Cursor extensions
fn scan_cursor_extensions() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();
    let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // macOS: ~/Library/Application Support/Cursor/User/extensions/
    let macos_path = home
        .join("Library")
        .join("Application Support")
        .join("Cursor")
        .join("User")
        .join("extensions");

    // Linux: ~/.cursor/extensions/
    let linux_path = home.join(".cursor").join("extensions");

    // Windows: %APPDATA%\Cursor\User\extensions\
    let windows_path = dirs_next::data_dir()
        .map(|d| d.join("Cursor").join("User").join("extensions"))
        .unwrap_or_else(|| PathBuf::from(""));

    for ext_dir in &[macos_path, linux_path, windows_path] {
        if ext_dir.exists() {
            packages.extend(scan_extension_directory(ext_dir, PackageSource::CursorExtension)?);
        }
    }

    Ok(packages)
}

/// Scan an extension directory for package.json files
fn scan_extension_directory(
    ext_dir: &Path,
    source: PackageSource,
) -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();

    if !ext_dir.exists() {
        return Ok(packages);
    }

    let entries = fs::read_dir(ext_dir)?;

    for entry in entries {
        let entry = entry?;
        let extension_path = entry.path();

        if extension_path.is_dir() {
            // Each extension has its own directory
            // Look for package.json in the extension root
            let package_json = extension_path.join("package.json");

            if package_json.exists() {
                // Parse the extension's package.json
                if let Ok(mut ext_packages) = parse_package_json(&package_json, source.clone()) {
                    // Also add the extension itself as a package
                    if let Ok(content) = fs::read_to_string(&package_json) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            let name = json
                                .get("name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("")
                                .to_string();
                            let version = json
                                .get("version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            if !name.is_empty() && !version.is_empty() {
                                ext_packages.push(FoundPackage {
                                    name,
                                    version,
                                    location: package_json.clone(),
                                    file_type: "VSCode/Cursor extension".to_string(),
                                    source: source.clone(),
                                });
                            }
                        }
                    }

                    packages.extend(ext_packages);
                }
            }
        }
    }

    Ok(packages)
}

