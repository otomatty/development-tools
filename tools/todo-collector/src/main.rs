//! TODO Collector
//!
//! A tool to collect and list TODO/FIXME/HACK/XXX/NOTE comments from codebase.

mod cli;
mod detector;
mod output;
mod scanner;
mod types;

use anyhow::Result;
use clap::Parser;
use colored::*;
use std::env;
use std::path::PathBuf;

use cli::Args;
use detector::detect_todos;
use output::output_results;
use scanner::scan_directory;
use types::{ScanResult, TodoItem};

fn main() -> Result<()> {
    let args = Args::parse();

    // Determine scan directory
    let scan_dir = determine_scan_directory(&args);
    let patterns = args.get_patterns();
    let excludes = args.get_excludes();

    // Print banner (only for text output)
    if args.output == cli::OutputFormat::Text {
        print_banner();
        println!(
            "{} Scanning directory: {}",
            "→".blue(),
            scan_dir.display()
        );
        println!(
            "{} Patterns: {}",
            "→".blue(),
            patterns.join(", ")
        );
        if !excludes.is_empty() {
            println!(
                "{} Excluding: {}",
                "→".blue(),
                excludes.join(", ")
            );
        }
        println!();
    }

    // Scan for source files
    let files = scan_directory(&scan_dir, &excludes)?;

    if args.output == cli::OutputFormat::Text {
        println!(
            "{} Found {} files to scan",
            "✓".green(),
            files.len()
        );
    }

    // Collect all TODOs
    let mut all_todos: Vec<TodoItem> = Vec::new();

    for file in &files {
        match detect_todos(file, &patterns) {
            Ok(todos) => all_todos.extend(todos),
            Err(e) => {
                if args.output == cli::OutputFormat::Text {
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

    if args.output == cli::OutputFormat::Text {
        println!(
            "{} Found {} TODO comments",
            "✓".green(),
            all_todos.len()
        );
    }

    // Create result
    let mut result = ScanResult::from_items(all_todos);

    // Sort by priority if requested
    if args.priority {
        result.sort_by_priority();
    }

    // Output results
    output_results(&result, args.output);

    Ok(())
}

/// Print the application banner
fn print_banner() {
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║                      TODO Collector                           ║".cyan()
    );
    println!(
        "{}",
        "║   Collect and list TODO/FIXME/HACK comments from codebase    ║".cyan()
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
    } else {
        // Default: current working directory
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }
}

