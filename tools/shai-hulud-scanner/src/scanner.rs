//! Local package scanning functionality

use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

use crate::parsers::{
    parse_bun_lock, parse_package_json, parse_package_lock_json, parse_pnpm_lock, parse_yarn_lock,
};
use crate::types::{FoundPackage, PackageSource};

/// Directories to skip during scanning
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    "build",
    ".next",
    "out",
    ".cache",
    ".npm",
    ".yarn",
    ".pnpm-store",
    "Applications",
    ".Trash",
    "Pictures",
    "Music",
    "Movies",
    "Downloads",
    // Skip Library subdirectories except Application Support
    "Caches",
    "Logs",
    "Preferences",
    "Saved Application State",
    "WebKit",
];

/// Scan directory for package files
pub fn scan_directory(dir: &Path) -> Result<Vec<FoundPackage>> {
    let mut found_packages = Vec::new();
    let skip_dirs: HashSet<&str> = SKIP_DIRS.iter().copied().collect();

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .max_depth(10) // Limit depth to avoid too deep recursion
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_str().unwrap_or("");
            // Skip dot-prefixed directories except .vscode and .cursor
            if file_name.starts_with('.') && file_name != ".vscode" && file_name != ".cursor" {
                return false;
            }
            // Skip directories in SKIP_DIRS
            !skip_dirs.contains(&file_name)
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // Skip permission errors, etc.
        };
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            match file_name {
                "package.json" => {
                    if let Ok(packages) = parse_package_json(path, PackageSource::Local) {
                        found_packages.extend(packages);
                    }
                }
                "package-lock.json" => {
                    if let Ok(packages) = parse_package_lock_json(path, PackageSource::Local) {
                        found_packages.extend(packages);
                    }
                }
                "bun.lock" | "bun.lockb" => {
                    if let Ok(packages) = parse_bun_lock(path, PackageSource::Local) {
                        found_packages.extend(packages);
                    }
                }
                "yarn.lock" => {
                    if let Ok(packages) = parse_yarn_lock(path, PackageSource::Local) {
                        found_packages.extend(packages);
                    }
                }
                "pnpm-lock.yaml" => {
                    if let Ok(packages) = parse_pnpm_lock(path, PackageSource::Local) {
                        found_packages.extend(packages);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(found_packages)
}

