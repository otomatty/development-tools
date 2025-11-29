//! Large File Finder
//!
//! A CLI tool to find files with more than a specified number of lines,
//! sorted by line count in descending order.
//!
//! Features:
//! - Automatically respects .gitignore patterns
//! - Supports text and JSON output formats
//! - Can limit results to top N files
//! - Built-in presets for common languages
//! - Configuration file support (.largefile.toml)

mod cli;
mod config;
mod counter;
mod output;
mod presets;
mod scanner;
mod types;

use anyhow::Result;
use clap::Parser;
use colored::*;

use cli::{Args, OutputFormat};
use config::{load_config, Config, CONFIG_FILE_NAME};
use counter::{count_lines, is_text_file};
use output::output_results;
use presets::{detect_presets, get_all_patterns, parse_presets, PresetName};
use scanner::scan_directory;
use types::{FileInfo, ScanResult};

fn main() -> Result<()> {
    let args = Args::parse();

    // Determine scan directory
    let scan_path = args.get_scan_path();

    // Validate scan path
    if !scan_path.exists() {
        eprintln!(
            "{} Directory does not exist: {}",
            "Error:".red().bold(),
            scan_path.display()
        );
        std::process::exit(1);
    }

    if !scan_path.is_dir() {
        eprintln!(
            "{} Path is not a directory: {}",
            "Error:".red().bold(),
            scan_path.display()
        );
        std::process::exit(1);
    }

    // Load configuration file if exists
    let config = match load_config(&scan_path) {
        Ok(cfg) => cfg.unwrap_or_default(),
        Err(e) => {
            if args.output == OutputFormat::Text {
                eprintln!("{} Warning: {}", "⚠".yellow(), e);
            }
            Config::default()
        }
    };
    let has_config_file = config.exclude.len() > 0 
        || config.min_lines.is_some() 
        || !config.presets.manual.is_empty()
        || !config.presets.auto_detect;

    // Determine min_lines (CLI > config > default)
    let min_lines = if args.min_lines != 500 {
        // CLI was explicitly set (non-default value)
        args.min_lines
    } else {
        config.min_lines.unwrap_or(args.min_lines)
    };

    // Determine presets to use (CLI > config > auto-detect)
    let presets = determine_presets(&args, &config, &scan_path);

    // Get all exclusion patterns from presets
    let mut excludes = get_all_patterns(&presets);

    // Add config file excludes
    excludes.extend(config.exclude.clone());

    // Add user-specified excludes from CLI
    excludes.extend(args.get_excludes());

    // Print banner (only for text output)
    if args.output == OutputFormat::Text {
        print_scanning_info(&args, &scan_path, &presets, &excludes, has_config_file, min_lines);
    }

    // Scan for files
    let files = scan_directory(&scan_path, &excludes)?;

    if args.output == OutputFormat::Text {
        println!("{} Found {} files to analyze", "✓".green(), files.len());
    }

    // Create result
    let mut result = ScanResult::new(
        scan_path.display().to_string(),
        min_lines,
    );

    // Add preset info to result
    result.presets = presets.iter().map(|p| p.as_str().to_string()).collect();

    // Count lines in each file
    for file_path in &files {
        result.increment_scanned();

        // Skip binary files
        if !is_text_file(file_path) {
            continue;
        }

        match count_lines(file_path) {
            Ok(lines) => {
                let relative_path = file_path
                    .strip_prefix(&scan_path)
                    .unwrap_or(file_path)
                    .display()
                    .to_string();

                let file_info = FileInfo::new(relative_path, file_path.clone(), lines);
                result.add_file(file_info);
            }
            Err(e) => {
                if args.output == OutputFormat::Text {
                    eprintln!(
                        "{} Error reading {}: {}",
                        "⚠".yellow(),
                        file_path.display(),
                        e
                    );
                }
            }
        }
    }

    // Sort by line count (descending)
    result.sort_by_lines();

    // Apply top limit if specified
    if let Some(top) = args.top {
        result.limit(top);
    }

    // Output results
    output_results(&result, args.output);

    Ok(())
}

/// Determine which presets to use based on CLI args, config file, and project detection
fn determine_presets(args: &Args, config: &Config, scan_path: &std::path::Path) -> Vec<PresetName> {
    // Priority 1: CLI specified presets
    if let Some(preset_str) = &args.preset {
        return parse_presets(preset_str);
    }

    // Priority 2: Config file manual presets
    if !config.presets.manual.is_empty() {
        return parse_presets(&config.presets.manual.join(","));
    }

    // Priority 3: Auto-detect if enabled (CLI flag takes precedence over config)
    let auto_detect_enabled = if args.no_auto_detect {
        false
    } else {
        config.presets.auto_detect
    };

    if auto_detect_enabled {
        return detect_presets(scan_path);
    }

    // Default: just use common preset
    vec![PresetName::Common]
}

/// Print scanning information (for text output)
fn print_scanning_info(
    args: &Args,
    scan_path: &std::path::Path,
    presets: &[PresetName],
    excludes: &[String],
    has_config_file: bool,
    min_lines: usize,
) {
    println!();
    println!(
        "{} Scanning directory: {}",
        "→".blue(),
        scan_path.display()
    );

    if has_config_file {
        println!(
            "{} Config: {} found",
            "→".blue(),
            CONFIG_FILE_NAME.cyan()
        );
    }

    println!(
        "{} Min lines threshold: {}",
        "→".blue(),
        min_lines
    );

    if let Some(top) = args.top {
        println!("{} Showing top {} files", "→".blue(), top);
    }

    // Show presets being used
    let preset_names: Vec<&str> = presets.iter().map(|p| p.as_str()).collect();
    let auto_detected = args.preset.is_none() && !args.no_auto_detect;
    println!(
        "{} Presets: {}{}",
        "→".blue(),
        preset_names.join(", ").cyan(),
        if auto_detected { " (auto-detected)".dimmed().to_string() } else { "".to_string() }
    );

    if !excludes.is_empty() && excludes.len() <= 10 {
        println!(
            "{} Excluding: {}",
            "→".blue(),
            excludes.join(", ").dimmed()
        );
    } else if !excludes.is_empty() {
        println!(
            "{} Excluding: {} patterns",
            "→".blue(),
            excludes.len()
        );
    }

    println!("{} Using .gitignore patterns", "→".blue());
    println!();
}
