//! Suspicious file detection for potential malware

use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::types::{Severity, SuspiciousFile};

/// Known suspicious patterns from Shai-Hulud attack
const SUSPICIOUS_PATTERNS: &[(&str, &str)] = &[
    ("bun", "Potential fake bun binary"),
    ("bunx", "Potential fake bunx binary"),
    ("trufflehog", "TruffleHog binary (used by Shai-Hulud)"),
    ("preinstall.sh", "Suspicious preinstall script"),
    ("postinstall.sh", "Suspicious postinstall script"),
];

/// Known safe packages that use postinstall legitimately
const SAFE_PACKAGES: &[&str] = &[
    // Build tools that download native binaries
    "@biomejs/biome",
    "@swc/core",
    "esbuild",
    "turbo",
    "lightningcss",
    // Polyfills that show donation messages
    "core-js",
    "core-js-pure",
    // CLI tools
    "supabase",
    "prisma",
    // Testing tools
    "msw",
    "playwright",
    "puppeteer",
    // Husky and git hooks (legitimate postinstall)
    "husky",
    "simple-git-hooks",
];

/// High confidence malicious patterns (Shai-Hulud specific)
const MALICIOUS_PATTERNS: &[&str] = &[
    "trufflehog",           // Used by Shai-Hulud to scan secrets
    "npm whoami",           // Used to get npm credentials
    "npm publish",          // Used to publish malicious packages
    "git push.*--force",    // Force pushing to repos
    "curl.*\\|.*sh",        // Curl piping to shell
    "wget.*\\|.*sh",        // Wget piping to shell
    "curl.*\\|.*bash",      // Curl piping to bash
    "eval.*\\$\\(",         // eval with command substitution
    "base64.*-d.*\\|",      // Base64 decode piping
    "\\bsecret\\b.*upload", // Uploading secrets
    "\\btoken\\b.*upload",  // Uploading tokens
    "github\\.com.*push",   // Pushing to GitHub
];

/// Directories to skip during scanning
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    "build",
    ".next",
    "out",
    "Applications",
    ".Trash",
    "Pictures",
    "Music",
    "Movies",
    // Skip Library subdirectories except Application Support
    "Caches",
    "Logs",
    "Preferences",
    "Saved Application State",
    "WebKit",
];

/// Detect suspicious files that might be malicious
pub fn detect_suspicious_files(dir: &Path) -> Result<Vec<SuspiciousFile>> {
    let mut suspicious = Vec::new();
    let skip_dirs: HashSet<&str> = SKIP_DIRS.iter().copied().collect();

    // Check home directory for suspicious binaries
    let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // Check common binary locations
    let bin_locations = [
        home.join(".local").join("bin"),
        home.join("bin"),
        home.join(".bun").join("bin"),
        PathBuf::from("/usr/local/bin"),
    ];

    for bin_dir in &bin_locations {
        if bin_dir.exists() {
            if let Ok(entries) = fs::read_dir(bin_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    for (pattern, reason) in SUSPICIOUS_PATTERNS {
                        if file_name == *pattern || file_name.starts_with(&format!("{}.", pattern))
                        {
                            // Check if this is a legitimate binary or suspicious
                            if is_suspicious_binary(&path) {
                                suspicious.push(SuspiciousFile {
                                    path: path.clone(),
                                    reason: reason.to_string(),
                                    severity: Severity::Critical,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Scan for suspicious postinstall scripts in the scan directory
    for entry in WalkDir::new(dir)
        .follow_links(false)
        .max_depth(8)
        .into_iter()
        .filter_entry(|e| {
            e.file_name()
                .to_str()
                .map(|s| !skip_dirs.contains(s))
                .unwrap_or(true)
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Check for suspicious postinstall scripts
            if file_name == "postinstall"
                || file_name == "postinstall.js"
                || file_name == "postinstall.sh"
            {
                if let Ok(content) = fs::read_to_string(path) {
                    // Check for suspicious patterns in the script
                    if contains_suspicious_content(&content) {
                        suspicious.push(SuspiciousFile {
                            path: path.to_path_buf(),
                            reason: "Suspicious postinstall script content".to_string(),
                            severity: Severity::Critical,
                        });
                    }
                }
            }
        }
    }

    Ok(suspicious)
}

/// Check if a binary is suspicious
fn is_suspicious_binary(path: &Path) -> bool {
    // Check file metadata
    if let Ok(metadata) = fs::metadata(path) {
        // Very small files might be scripts pretending to be binaries
        if metadata.len() < 100 {
            return true;
        }

        // Check if it's a script (text file) pretending to be a binary
        if let Ok(content) = fs::read_to_string(path) {
            let lower = content.to_lowercase();
            // Check for suspicious patterns
            if lower.contains("curl") && lower.contains("eval") {
                return true;
            }
            if lower.contains("trufflehog") {
                return true;
            }
            if lower.contains("npm publish") || lower.contains("npm whoami") {
                return true;
            }
        }
    }

    false
}

/// Check if content contains suspicious patterns
fn contains_suspicious_content(content: &str) -> bool {
    let lower = content.to_lowercase();

    // If the script mentions any safe package, skip it
    for safe in SAFE_PACKAGES {
        if lower.contains(safe) {
            return false;
        }
    }

    // Check for high confidence malicious patterns
    for pattern in MALICIOUS_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&lower) {
                return true;
            }
        }
    }

    false
}

