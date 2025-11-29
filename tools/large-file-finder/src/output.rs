//! Output formatting for Large File Finder
//!
//! This module handles text and JSON output formatting.

use colored::*;

use crate::cli::OutputFormat;
use crate::types::ScanResult;

/// Output the scan results in the specified format
pub fn output_results(result: &ScanResult, format: OutputFormat) {
    match format {
        OutputFormat::Text => output_text(result),
        OutputFormat::Json => output_json(result),
    }
}

/// Output results as formatted text with colors
fn output_text(result: &ScanResult) {
    let separator = "━".repeat(60);

    println!();
    println!("{}", "🔍 Large File Finder".bold());
    println!("{}", separator.dimmed());
    println!();
    println!(
        "{} Scanning: {}",
        "→".blue(),
        result.scan_directory.cyan()
    );
    println!(
        "{} Min lines: {}",
        "→".blue(),
        result.min_lines.to_string().yellow()
    );
    println!(
        "{} Excluding: .gitignore patterns",
        "→".blue()
    );
    println!();
    println!("{}", separator.dimmed());
    println!();

    if result.files.is_empty() {
        println!(
            "{} No files found with {}+ lines",
            "✓".green(),
            result.min_lines
        );
    } else {
        println!(
            "Found {} {} with {}+ lines:",
            result.files.len().to_string().bold().yellow(),
            if result.files.len() == 1 { "file" } else { "files" },
            result.min_lines
        );
        println!();

        // Calculate max line count width for alignment
        let max_lines_width = result
            .files
            .iter()
            .map(|f| f.lines.to_string().len())
            .max()
            .unwrap_or(1);

        // Severity thresholds for color coding
        const SEVERE_THRESHOLD: usize = 1000;
        const WARN_THRESHOLD: usize = 750;

        for (i, file) in result.files.iter().enumerate() {
            let rank = format!("{:>3}.", i + 1);
            let lines = format!("{:>width$}", file.lines, width = max_lines_width);
            
            // Color code based on severity
            let lines_colored = if file.lines >= SEVERE_THRESHOLD {
                lines.red().bold()
            } else if file.lines >= WARN_THRESHOLD {
                lines.yellow()
            } else {
                lines.white()
            };

            println!(
                "  {} {} lines  {}",
                rank.dimmed(),
                lines_colored,
                file.path.cyan()
            );
        }
    }

    println!();
    println!("{}", separator.dimmed());
    
    // Summary
    println!(
        "Total: {} {}, {} lines combined",
        result.files.len().to_string().bold(),
        if result.files.len() == 1 { "file" } else { "files" },
        format_number(result.total_lines).bold()
    );
    println!(
        "Scanned: {} files",
        format_number(result.total_files_scanned)
    );
}

/// Output results as JSON
fn output_json(result: &ScanResult) {
    match serde_json::to_string_pretty(result) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

/// Format a number with thousand separators
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(100), "100");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }
}
