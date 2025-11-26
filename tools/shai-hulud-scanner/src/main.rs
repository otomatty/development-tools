//! Shai-Hulud Scanner
//!
//! A security tool to detect npm packages affected by the Shai-Hulud supply chain attack.
//!
//! Reference:
//! - https://codebook.machinarecord.com/threatreport/silobreaker-cyber-alert/42718/
//! - https://github.com/wiz-sec-public/wiz-research-iocs/blob/main/reports/shai-hulud-2-packages.csv

mod cli;
mod csv_loader;
mod detector;
mod global_scanner;
mod output;
mod parsers;
mod scanner;
mod suspicious;
mod types;

use anyhow::Result;
use clap::Parser;
use colored::*;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use cli::Args;
use csv_loader::load_affected_packages;
use detector::detect_affected_packages;
use global_scanner::scan_global_packages;
use output::{output_results, print_summary};
use scanner::scan_directory;
use suspicious::detect_suspicious_files;

fn main() -> Result<()> {
    let args = Args::parse();

    print_banner();

    // Determine scan directory
    let scan_dir = determine_scan_directory(&args);

    // Load affected packages from CSV
    let affected_packages = load_affected_packages(&args)?;
    println!(
        "{} {} affected packages loaded from CSV",
        "✓".green(),
        affected_packages.len()
    );

    // Create a lookup map for faster searching
    let affected_map: HashMap<String, &types::AffectedPackage> = affected_packages
        .iter()
        .map(|p| (p.name.clone(), p))
        .collect();

    // Scan for package files
    println!(
        "\n{} Scanning directory: {}",
        "→".blue(),
        scan_dir.display()
    );
    let mut found_packages = scan_directory(&scan_dir)?;
    println!(
        "{} Found {} package files in local projects",
        "✓".green(),
        found_packages.len()
    );

    // Scan global packages
    if !args.skip_global {
        println!("\n{} Scanning global packages...", "→".blue());
        let global_packages = scan_global_packages()?;
        println!(
            "{} Found {} global packages (npm/yarn/pnpm/bun)",
            "✓".green(),
            global_packages.len()
        );
        found_packages.extend(global_packages);
    }

    // Detect affected packages
    let detections = detect_affected_packages(&found_packages, &affected_map);

    // Detect suspicious files
    let suspicious_files = if !args.skip_suspicious {
        println!("\n{} Scanning for suspicious files...", "→".blue());
        let files = detect_suspicious_files(&scan_dir)?;
        println!("{} Checked for suspicious patterns", "✓".green());
        files
    } else {
        Vec::new()
    };

    // Output results
    println!();
    output_results(&detections, &suspicious_files, &args);

    // Summary
    print_summary(&detections, &suspicious_files);

    Ok(())
}

/// Print the application banner
fn print_banner() {
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║         Shai-Hulud Supply Chain Attack Scanner               ║".cyan()
    );
    println!(
        "{}",
        "║   Detecting affected npm packages in your local environment  ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();
}

/// Determine which directory to scan
fn determine_scan_directory(args: &Args) -> PathBuf {
    if let Some(ref dir) = args.scan_dir {
        dir.clone()
    } else if args.current_dir {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else {
        // Default: scan home directory for full PC coverage
        dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."))
    }
}
