//! CSV loading and parsing for affected packages list

use anyhow::{Context, Result};
use colored::*;
use regex::Regex;
use std::fs;
use std::path::PathBuf;

use crate::cli::Args;
use crate::types::{AffectedPackage, VersionConstraint};

/// Load affected packages from CSV (either from file or URL)
pub fn load_affected_packages(args: &Args) -> Result<Vec<AffectedPackage>> {
    let csv_content = if let Some(ref csv_path) = args.csv_file {
        println!(
            "{} Loading CSV from local file: {}",
            "→".blue(),
            csv_path.display()
        );
        fs::read_to_string(csv_path).context("Failed to read local CSV file")?
    } else if args.offline {
        // Try to load from cache
        let cache_path = get_cache_path();
        if cache_path.exists() {
            println!(
                "{} Loading CSV from cache: {}",
                "→".blue(),
                cache_path.display()
            );
            fs::read_to_string(&cache_path).context("Failed to read cached CSV file")?
        } else {
            anyhow::bail!(
                "Offline mode requested but no cached CSV found. Run without --offline first."
            );
        }
    } else {
        println!("{} Downloading CSV from: {}", "→".blue(), args.csv_url);
        let content = download_csv(&args.csv_url)?;

        // Cache the downloaded CSV
        if let Err(e) = cache_csv(&content) {
            eprintln!("{} Failed to cache CSV: {}", "⚠".yellow(), e);
        }

        content
    };

    parse_csv(&csv_content)
}

/// Download CSV from URL
fn download_csv(url: &str) -> Result<String> {
    let response = reqwest::blocking::get(url).context("Failed to download CSV from URL")?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP error: {}", response.status());
    }

    response.text().context("Failed to read response body")
}

/// Get cache file path
fn get_cache_path() -> PathBuf {
    dirs_next::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("shai-hulud-scanner")
        .join("affected-packages.csv")
}

/// Cache the CSV content
fn cache_csv(content: &str) -> Result<()> {
    let cache_path = get_cache_path();
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&cache_path, content)?;
    Ok(())
}

/// Parse CSV content into affected packages
fn parse_csv(content: &str) -> Result<Vec<AffectedPackage>> {
    let mut reader = csv::Reader::from_reader(content.as_bytes());
    let mut packages = Vec::new();

    for result in reader.records() {
        let record = result.context("Failed to parse CSV record")?;

        if record.len() < 2 {
            continue;
        }

        let name = record.get(0).unwrap_or("").trim().to_string();
        let version_str = record.get(1).unwrap_or("").trim();

        if name.is_empty() || name == "Package" {
            continue; // Skip header or empty rows
        }

        let versions = parse_version_constraints(version_str);

        packages.push(AffectedPackage { name, versions });
    }

    Ok(packages)
}

/// Parse version constraints like "= 0.0.7 || = 0.0.8"
fn parse_version_constraints(s: &str) -> Vec<VersionConstraint> {
    let re = Regex::new(r"=\s*(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?)").unwrap();

    re.captures_iter(s)
        .map(|cap| VersionConstraint {
            version: cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default(),
        })
        .collect()
}

