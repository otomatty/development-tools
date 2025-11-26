//! File parsers for various package manager lockfiles

use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

use crate::types::{FoundPackage, PackageJson, PackageLockJson, PackageSource};

/// Parse package.json file
pub fn parse_package_json(path: &Path, source: PackageSource) -> Result<Vec<FoundPackage>> {
    let content = fs::read_to_string(path)?;
    let pkg: PackageJson = serde_json::from_str(&content)?;

    let mut packages = Vec::new();
    let location = path.to_path_buf();

    for (name, version) in pkg
        .dependencies
        .iter()
        .chain(pkg.dev_dependencies.iter())
        .chain(pkg.optional_dependencies.iter())
    {
        packages.push(FoundPackage {
            name: name.clone(),
            version: clean_version(version),
            location: location.clone(),
            file_type: "package.json".to_string(),
            source: source.clone(),
        });
    }

    Ok(packages)
}

/// Parse package-lock.json file
pub fn parse_package_lock_json(path: &Path, source: PackageSource) -> Result<Vec<FoundPackage>> {
    let content = fs::read_to_string(path)?;
    let lock: PackageLockJson = serde_json::from_str(&content)?;

    let mut packages = Vec::new();
    let location = path.to_path_buf();

    // Parse packages (npm v7+)
    for (key, entry) in &lock.packages {
        if let Some(ref version) = entry.version {
            let name = key.strip_prefix("node_modules/").unwrap_or(key);
            if !name.is_empty() {
                packages.push(FoundPackage {
                    name: name.to_string(),
                    version: version.clone(),
                    location: location.clone(),
                    file_type: "package-lock.json".to_string(),
                    source: source.clone(),
                });
            }
        }
    }

    // Parse dependencies (npm v6)
    for (name, dep) in &lock.dependencies {
        if let Some(ref version) = dep.version {
            packages.push(FoundPackage {
                name: name.clone(),
                version: version.clone(),
                location: location.clone(),
                file_type: "package-lock.json".to_string(),
                source: source.clone(),
            });
        }
    }

    Ok(packages)
}

/// Parse bun.lock file
pub fn parse_bun_lock(path: &Path, source: PackageSource) -> Result<Vec<FoundPackage>> {
    let content = fs::read_to_string(path)?;
    let location = path.to_path_buf();
    let mut packages = Vec::new();

    // Try to parse as JSON first
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(pkgs) = json.get("packages").and_then(|p| p.as_object()) {
            for (key, value) in pkgs {
                let name = key.clone();
                let version = value
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_default();

                if !name.is_empty() && !version.is_empty() {
                    packages.push(FoundPackage {
                        name,
                        version,
                        location: location.clone(),
                        file_type: "bun.lock".to_string(),
                        source: source.clone(),
                    });
                }
            }
        }
    } else {
        // Parse as text format (package@version)
        let re = Regex::new(r#""([^"@]+)@([^"]+)""#).unwrap();
        for cap in re.captures_iter(&content) {
            let name = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let version = cap
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            if !name.is_empty() && !version.is_empty() {
                packages.push(FoundPackage {
                    name,
                    version: clean_version(&version),
                    location: location.clone(),
                    file_type: "bun.lock".to_string(),
                    source: source.clone(),
                });
            }
        }
    }

    Ok(packages)
}

/// Parse yarn.lock file
pub fn parse_yarn_lock(path: &Path, source: PackageSource) -> Result<Vec<FoundPackage>> {
    let content = fs::read_to_string(path)?;
    let location = path.to_path_buf();
    let mut packages = Vec::new();

    // Parse yarn.lock format
    // Format: "package@version":
    //   version "x.x.x"
    let package_re = Regex::new(r#"^"?([^@"\s]+)@[^"]*"?:$"#).unwrap();
    let version_re = Regex::new(r#"^\s+version\s+"([^"]+)""#).unwrap();

    let mut current_package: Option<String> = None;

    for line in content.lines() {
        if let Some(cap) = package_re.captures(line) {
            current_package = cap.get(1).map(|m| m.as_str().to_string());
        } else if let (Some(ref pkg), Some(cap)) = (&current_package, version_re.captures(line)) {
            let version = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            packages.push(FoundPackage {
                name: pkg.clone(),
                version,
                location: location.clone(),
                file_type: "yarn.lock".to_string(),
                source: source.clone(),
            });
            current_package = None;
        }
    }

    Ok(packages)
}

/// Parse pnpm-lock.yaml file
pub fn parse_pnpm_lock(path: &Path, source: PackageSource) -> Result<Vec<FoundPackage>> {
    let content = fs::read_to_string(path)?;
    let location = path.to_path_buf();
    let mut packages = Vec::new();

    // Simple regex-based parsing for pnpm-lock.yaml
    // Format: /package@version: or package@version:
    let re = Regex::new(r"/?([^@/\s]+)@(\d+\.\d+\.\d+[^:\s]*)").unwrap();

    for cap in re.captures_iter(&content) {
        let name = cap
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let version = cap
            .get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        if !name.is_empty() && !version.is_empty() && !name.starts_with('@') {
            packages.push(FoundPackage {
                name,
                version,
                location: location.clone(),
                file_type: "pnpm-lock.yaml".to_string(),
                source: source.clone(),
            });
        }
    }

    // Also handle scoped packages (@scope/package)
    let scoped_re = Regex::new(r"(@[^@/\s]+/[^@/\s]+)@(\d+\.\d+\.\d+[^:\s]*)").unwrap();
    for cap in scoped_re.captures_iter(&content) {
        let name = cap
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let version = cap
            .get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        if !name.is_empty() && !version.is_empty() {
            packages.push(FoundPackage {
                name,
                version,
                location: location.clone(),
                file_type: "pnpm-lock.yaml".to_string(),
                source: source.clone(),
            });
        }
    }

    Ok(packages)
}

/// Clean version string (remove ^, ~, >=, etc.)
pub fn clean_version(version: &str) -> String {
    let re = Regex::new(r"[\^~>=<]").unwrap();
    re.replace_all(version, "").trim().to_string()
}

