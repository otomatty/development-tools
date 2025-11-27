//! Global package scanning for npm, yarn, pnpm, and bun

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::parsers::parse_package_json;
use crate::types::{FoundPackage, PackageSource};

/// Scan global packages from all package managers
pub fn scan_global_packages() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();

    // npm global
    if let Ok(npm_packages) = scan_npm_global() {
        packages.extend(npm_packages);
    }

    // yarn global
    if let Ok(yarn_packages) = scan_yarn_global() {
        packages.extend(yarn_packages);
    }

    // pnpm global
    if let Ok(pnpm_packages) = scan_pnpm_global() {
        packages.extend(pnpm_packages);
    }

    // bun global
    if let Ok(bun_packages) = scan_bun_global() {
        packages.extend(bun_packages);
    }

    Ok(packages)
}

/// Scan npm global packages
fn scan_npm_global() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();

    // Try to get npm global prefix
    let output = Command::new("npm").args(["root", "-g"]).output();

    if let Ok(output) = output {
        if output.status.success() {
            let global_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let global_dir = PathBuf::from(&global_path);

            if global_dir.exists() {
                packages.extend(scan_global_dir(&global_dir, PackageSource::GlobalNpm)?);
            }
        }
    }

    Ok(packages)
}

/// Scan yarn global packages
fn scan_yarn_global() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();

    // Try to get yarn global dir
    let output = Command::new("yarn").args(["global", "dir"]).output();

    if let Ok(output) = output {
        if output.status.success() {
            let global_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let global_dir = PathBuf::from(&global_path).join("node_modules");

            if global_dir.exists() {
                packages.extend(scan_global_dir(&global_dir, PackageSource::GlobalYarn)?);
            }
        }
    }

    Ok(packages)
}

/// Scan pnpm global packages
fn scan_pnpm_global() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();

    // Try to get pnpm global dir
    let output = Command::new("pnpm").args(["root", "-g"]).output();

    if let Ok(output) = output {
        if output.status.success() {
            let global_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let global_dir = PathBuf::from(&global_path);

            if global_dir.exists() {
                packages.extend(scan_global_dir(&global_dir, PackageSource::GlobalPnpm)?);
            }
        }
    }

    Ok(packages)
}

/// Scan bun global packages
fn scan_bun_global() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();

    // Bun global packages are typically in ~/.bun/install/global
    let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let bun_global_dir = home
        .join(".bun")
        .join("install")
        .join("global")
        .join("node_modules");

    if bun_global_dir.exists() {
        packages.extend(scan_global_dir(&bun_global_dir, PackageSource::GlobalBun)?);
    }

    Ok(packages)
}

/// Scan a global directory for packages
fn scan_global_dir(dir: &PathBuf, source: PackageSource) -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();
    let file_type = match source {
        PackageSource::GlobalNpm => "npm global",
        PackageSource::GlobalYarn => "yarn global",
        PackageSource::GlobalPnpm => "pnpm global",
        PackageSource::GlobalBun => "bun global",
        PackageSource::Local => "local",
        _ => "package.json", // Other sources are handled elsewhere
    };

    for entry in fs::read_dir(dir).into_iter().flatten() {
        if let Ok(entry) = entry {
            let pkg_json = entry.path().join("package.json");
            if pkg_json.exists() {
                // Get the package itself (name and version from package.json)
                if let Ok(content) = fs::read_to_string(&pkg_json) {
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
                            packages.push(FoundPackage {
                                name,
                                version,
                                location: pkg_json.clone(),
                                file_type: file_type.to_string(),
                                source: source.clone(),
                            });
                        }
                    }
                }

                // Also scan dependencies
                if let Ok(pkgs) = parse_package_json(&pkg_json, source.clone()) {
                    packages.extend(pkgs);
                }
            }
        }
    }

    Ok(packages)
}

