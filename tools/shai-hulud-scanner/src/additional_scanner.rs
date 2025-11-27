//! Additional scanning functionality for Electron apps, Node version managers, and other locations

use anyhow::Result;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::parsers::parse_package_json;
use crate::types::{FoundPackage, PackageSource};

/// Scan all additional locations (Electron apps, Node version managers, etc.)
pub fn scan_additional_locations() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();

    // Scan Electron applications
    if let Ok(electron_packages) = scan_electron_apps() {
        packages.extend(electron_packages);
    }

    // Scan Node.js version managers
    if let Ok(nvm_packages) = scan_node_version_managers() {
        packages.extend(nvm_packages);
    }

    // Scan other IDE extensions
    if let Ok(ide_packages) = scan_other_ide_extensions() {
        packages.extend(ide_packages);
    }

    // Scan npm cache
    if let Ok(cache_packages) = scan_npm_cache() {
        packages.extend(cache_packages);
    }

    // Scan CI/CD tool caches
    if let Ok(cicd_packages) = scan_cicd_caches() {
        packages.extend(cicd_packages);
    }

    // Scan system package manager installations
    if let Ok(system_packages) = scan_system_package_managers() {
        packages.extend(system_packages);
    }

    Ok(packages)
}

/// Scan Electron applications for node_modules
fn scan_electron_apps() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();
    let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // Common Electron app names
    let electron_apps = [
        "Slack",
        "Discord",
        "Notion",
        "Figma",
        "Spotify",
        "WhatsApp",
        "Telegram",
        "Signal",
        "Microsoft Teams",
        "Zoom",
        "Postman",
        "Insomnia",
        "Hyper",
        "Atom",
        "Code",
        "Cursor",
    ];

    // macOS: ~/Library/Application Support/{AppName}/
    let macos_app_support = home.join("Library").join("Application Support");

    // Windows: %APPDATA%\{AppName}\
    let windows_app_data = dirs_next::data_dir().unwrap_or_else(|| PathBuf::from(""));

    // Linux: ~/.config/{AppName}/
    let linux_config = home.join(".config");

    for app_name in &electron_apps {
        // macOS
        let macos_path = macos_app_support.join(app_name);
        if macos_path.exists() {
            packages.extend(scan_directory_for_package_json(&macos_path, PackageSource::ElectronApp)?);
        }

        // Windows
        let windows_path = windows_app_data.join(app_name);
        if windows_path.exists() {
            packages.extend(scan_directory_for_package_json(&windows_path, PackageSource::ElectronApp)?);
        }

        // Linux
        let linux_path = linux_config.join(app_name);
        if linux_path.exists() {
            packages.extend(scan_directory_for_package_json(&linux_path, PackageSource::ElectronApp)?);
        }
    }

    Ok(packages)
}

/// Scan Node.js version managers (nvm, n, fnm, volta)
fn scan_node_version_managers() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();
    let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // nvm: ~/.nvm/versions/node/{version}/lib/node_modules/
    let nvm_path = home.join(".nvm").join("versions").join("node");
    if nvm_path.exists() {
        if let Ok(entries) = fs::read_dir(&nvm_path) {
            for entry in entries.flatten() {
                let version_path = entry.path();
                let node_modules = version_path.join("lib").join("node_modules");
                if node_modules.exists() {
                    packages.extend(scan_directory_for_package_json(&node_modules, PackageSource::NodeVersionManager)?);
                }
            }
        }
    }

    // n: /usr/local/n/versions/node/{version}/lib/node_modules/ or ~/.n/versions/node/{version}/lib/node_modules/
    let n_paths = [
        PathBuf::from("/usr/local/n/versions/node"),
        home.join(".n").join("versions").join("node"),
    ];
    for n_path in &n_paths {
        if n_path.exists() {
            if let Ok(entries) = fs::read_dir(n_path) {
                for entry in entries.flatten() {
                    let version_path = entry.path();
                    let node_modules = version_path.join("lib").join("node_modules");
                    if node_modules.exists() {
                        packages.extend(scan_directory_for_package_json(&node_modules, PackageSource::NodeVersionManager)?);
                    }
                }
            }
        }
    }

    // fnm: ~/.fnm/node-versions/{version}/lib/node_modules/
    let fnm_path = home.join(".fnm").join("node-versions");
    if fnm_path.exists() {
        if let Ok(entries) = fs::read_dir(&fnm_path) {
            for entry in entries.flatten() {
                let version_path = entry.path();
                let node_modules = version_path.join("lib").join("node_modules");
                if node_modules.exists() {
                    packages.extend(scan_directory_for_package_json(&node_modules, PackageSource::NodeVersionManager)?);
                }
            }
        }
    }

    // volta: ~/.volta/tools/image/node/{version}/lib/node_modules/
    let volta_path = home.join(".volta").join("tools").join("image").join("node");
    if volta_path.exists() {
        if let Ok(entries) = fs::read_dir(&volta_path) {
            for entry in entries.flatten() {
                let version_path = entry.path();
                let node_modules = version_path.join("lib").join("node_modules");
                if node_modules.exists() {
                    packages.extend(scan_directory_for_package_json(&node_modules, PackageSource::NodeVersionManager)?);
                }
            }
        }
    }

    Ok(packages)
}

/// Scan other IDE extensions (WebStorm, IntelliJ IDEA, Atom, Sublime Text, etc.)
fn scan_other_ide_extensions() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();
    let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // WebStorm/IntelliJ IDEA: ~/.IntelliJIdea{version}/config/plugins/
    let idea_base = home.join(".IntelliJIdea");
    if idea_base.exists() {
        if let Ok(entries) = fs::read_dir(&idea_base) {
            for entry in entries.flatten() {
                let plugins_path = entry.path().join("config").join("plugins");
                if plugins_path.exists() {
                    packages.extend(scan_directory_for_package_json(&plugins_path, PackageSource::OtherIDE)?);
                }
            }
        }
    }

    // JetBrains IDEs: ~/Library/Application Support/JetBrains/{IDE}/plugins/
    let macos_jetbrains = home
        .join("Library")
        .join("Application Support")
        .join("JetBrains");
    if macos_jetbrains.exists() {
        if let Ok(entries) = fs::read_dir(&macos_jetbrains) {
            for entry in entries.flatten() {
                let plugins_path = entry.path().join("plugins");
                if plugins_path.exists() {
                    packages.extend(scan_directory_for_package_json(&plugins_path, PackageSource::OtherIDE)?);
                }
            }
        }
    }

    // Atom: ~/.atom/packages/
    let atom_path = home.join(".atom").join("packages");
    if atom_path.exists() {
        packages.extend(scan_directory_for_package_json(&atom_path, PackageSource::OtherIDE)?);
    }

    // Sublime Text: ~/Library/Application Support/Sublime Text/Packages/
    let sublime_path = home
        .join("Library")
        .join("Application Support")
        .join("Sublime Text")
        .join("Packages");
    if sublime_path.exists() {
        packages.extend(scan_directory_for_package_json(&sublime_path, PackageSource::OtherIDE)?);
    }

    // Windows JetBrains: %APPDATA%\JetBrains\{IDE}\plugins\
    let windows_jetbrains = dirs_next::data_dir()
        .map(|d| d.join("JetBrains"))
        .unwrap_or_else(|| PathBuf::from(""));
    if windows_jetbrains.exists() {
        if let Ok(entries) = fs::read_dir(&windows_jetbrains) {
            for entry in entries.flatten() {
                let plugins_path = entry.path().join("plugins");
                if plugins_path.exists() {
                    packages.extend(scan_directory_for_package_json(&plugins_path, PackageSource::OtherIDE)?);
                }
            }
        }
    }

    Ok(packages)
}

/// Scan npm cache directory
fn scan_npm_cache() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();
    let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // npm cache: ~/.npm/_cacache/
    let npm_cache = home.join(".npm").join("_cacache");
    if npm_cache.exists() {
        // npm cache contains content-v2 and index-v5 directories
        // We'll scan for package.json files in the cache
        packages.extend(scan_directory_for_package_json(&npm_cache, PackageSource::NpmCache)?);
    }

    Ok(packages)
}

/// Scan CI/CD tool local caches
fn scan_cicd_caches() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();
    let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // GitHub Actions runner: ~/actions-runner/_work/
    let github_actions_path = home.join("actions-runner").join("_work");
    if github_actions_path.exists() {
        packages.extend(scan_directory_for_package_json(&github_actions_path, PackageSource::CICD)?);
    }

    // CircleCI: ~/.circleci/
    let circleci_path = home.join(".circleci");
    if circleci_path.exists() {
        packages.extend(scan_directory_for_package_json(&circleci_path, PackageSource::CICD)?);
    }

    // GitLab Runner: ~/gitlab-runner/builds/
    let gitlab_runner_path = home.join("gitlab-runner").join("builds");
    if gitlab_runner_path.exists() {
        packages.extend(scan_directory_for_package_json(&gitlab_runner_path, PackageSource::CICD)?);
    }

    Ok(packages)
}

/// Scan system package manager installations
fn scan_system_package_managers() -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();

    // Homebrew (macOS): /opt/homebrew/lib/node_modules/ or /usr/local/lib/node_modules/
    let homebrew_paths = [
        PathBuf::from("/opt/homebrew/lib/node_modules"),
        PathBuf::from("/usr/local/lib/node_modules"),
    ];
    for homebrew_path in &homebrew_paths {
        if homebrew_path.exists() {
            packages.extend(scan_directory_for_package_json(homebrew_path, PackageSource::SystemPackageManager)?);
        }
    }

    // Linux apt/yum: /usr/lib/node_modules/
    let linux_path = PathBuf::from("/usr/lib/node_modules");
    if linux_path.exists() {
        packages.extend(scan_directory_for_package_json(&linux_path, PackageSource::SystemPackageManager)?);
    }

    Ok(packages)
}

/// Scan a directory recursively for package.json files
fn scan_directory_for_package_json(
    dir: &Path,
    source: PackageSource,
) -> Result<Vec<FoundPackage>> {
    let mut packages = Vec::new();

    if !dir.exists() {
        return Ok(packages);
    }

    // Use WalkDir to recursively scan, but limit depth and skip node_modules
    for entry in WalkDir::new(dir)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_str().unwrap_or("");
            // Skip node_modules to avoid duplicate scanning
            file_name != "node_modules"
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_str().unwrap_or("");
            if file_name == "package.json" {
                if let Ok(mut pkgs) = parse_package_json(entry.path(), source.clone()) {
                    // Also add the package itself
                    if let Ok(content) = fs::read_to_string(entry.path()) {
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
                                let file_type = match source {
                                    PackageSource::ElectronApp => "Electron app",
                                    PackageSource::NodeVersionManager => "Node version manager",
                                    PackageSource::OtherIDE => "Other IDE extension",
                                    PackageSource::NpmCache => "npm cache",
                                    PackageSource::CICD => "CI/CD cache",
                                    PackageSource::SystemPackageManager => "System package manager",
                                    _ => "package.json",
                                };
                                pkgs.push(FoundPackage {
                                    name,
                                    version,
                                    location: entry.path().to_path_buf(),
                                    file_type: file_type.to_string(),
                                    source: source.clone(),
                                });
                            }
                        }
                    }
                    packages.extend(pkgs);
                }
            }
        }
    }

    Ok(packages)
}

