//! LOC Counter
//!
//! A tool to count lines of code by language with code/comment/blank line statistics.

mod cli;
mod counter;
mod language;
mod output;
mod scanner;
mod types;

use anyhow::Result;
use clap::Parser;
use colored::*;
use std::env;
use std::path::PathBuf;

use cli::{Args, OutputFormat};
use counter::count_file;
use output::output_results;
use scanner::scan_directory;
use types::{FileStats, LocResult};

fn main() -> Result<()> {
    let args = Args::parse();

    // Determine scan directory
    let scan_dir = determine_scan_directory(&args);
    let excludes = args.get_excludes();

    // Print banner (only for text output)
    if args.output == OutputFormat::Text {
        print_banner();
        println!(
            "{} Scanning directory: {}",
            "→".blue(),
            scan_dir.display()
        );
        if !excludes.is_empty() {
            println!("{} Excluding: {}", "→".blue(), excludes.join(", "));
        }
        println!();
    }

    // Scan for source files
    let files = scan_directory(&scan_dir, &excludes)?;

    if args.output == OutputFormat::Text {
        println!("{} Found {} files to analyze", "✓".green(), files.len());
    }

    // Count lines in each file
    let mut file_stats: Vec<FileStats> = Vec::new();

    for file in &files {
        match count_file(file) {
            Ok(stats) => file_stats.push(stats),
            Err(e) => {
                if args.output == OutputFormat::Text {
                    eprintln!(
                        "{} Error reading {}: {}",
                        "⚠".yellow(),
                        file.display(),
                        e
                    );
                }
            }
        }
    }

    if args.output == OutputFormat::Text {
        println!(
            "{} Analyzed {} files successfully",
            "✓".green(),
            file_stats.len()
        );
        println!();
    }

    // Create result
    let mut result = LocResult::from_file_stats(file_stats, args.by_file);

    // Sort by specified field
    result.sort_by(args.sort.as_str());

    // Output results
    output_results(&result, args.output);

    Ok(())
}

/// Determine the scan directory from arguments or use current directory
fn determine_scan_directory(args: &Args) -> PathBuf {
    match &args.scan_dir {
        Some(dir) => {
            // Expand ~ to home directory
            if dir.starts_with("~") {
                if let Some(home) = dirs_home() {
                    let rest = dir.strip_prefix("~").unwrap();
                    return home.join(rest.strip_prefix("/").unwrap_or(rest));
                }
            }
            dir.clone()
        }
        None => env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    }
}

/// Get home directory
fn dirs_home() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

/// Print banner
fn print_banner() {
    println!();
    println!(
        "{}",
        r#"
  _     ___   ____    ____                  _            
 | |   / _ \ / ___|  / ___|___  _   _ _ __ | |_ ___ _ __ 
 | |  | | | | |     | |   / _ \| | | | '_ \| __/ _ \ '__|
 | |__| |_| | |___  | |__| (_) | |_| | | | | ||  __/ |   
 |_____\___/ \____|  \____\___/ \__,_|_| |_|\__\___|_|   
"#
        .cyan()
    );
    println!(
        "  {} {}",
        "Version".dimmed(),
        env!("CARGO_PKG_VERSION").cyan()
    );
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_scan_directory_with_arg() {
        let args = Args {
            scan_dir: Some(PathBuf::from("/tmp/test")),
            exclude: String::new(),
            output: OutputFormat::Json,
            sort: cli::SortOrder::Lines,
            by_file: false,
        };

        let dir = determine_scan_directory(&args);
        assert_eq!(dir, PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_determine_scan_directory_without_arg() {
        let args = Args {
            scan_dir: None,
            exclude: String::new(),
            output: OutputFormat::Json,
            sort: cli::SortOrder::Lines,
            by_file: false,
        };

        let dir = determine_scan_directory(&args);
        // Should be current directory
        assert!(dir.exists() || dir == PathBuf::from("."));
    }
}
