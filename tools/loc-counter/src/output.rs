//! Output formatting for LOC Counter
//!
//! This module handles text and JSON output formatting.

use colored::*;

use crate::cli::OutputFormat;
use crate::types::LocResult;

/// Output results in the specified format
pub fn output_results(result: &LocResult, format: OutputFormat) {
    match format {
        OutputFormat::Json => output_json(result),
        OutputFormat::Text => output_text(result),
    }
}

/// Output results as JSON
fn output_json(result: &LocResult) {
    match serde_json::to_string_pretty(result) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing JSON: {}", e),
    }
}

/// Output results as formatted text
fn output_text(result: &LocResult) {
    print_header();
    print_summary(&result.summary);
    print_language_table(&result.by_language);

    if let Some(ref files) = result.files {
        print_file_details(files);
    }
}

/// Print header banner
fn print_header() {
    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║                     LOC Counter Results                          ║".cyan()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();
}

/// Print summary section
fn print_summary(summary: &crate::types::Summary) {
    println!("{}", "📊 Summary".bold().white());
    println!("{}", "─".repeat(40).dimmed());
    println!(
        "  {} {:>10}",
        "Total Files:".white(),
        format_number(summary.total_files).cyan()
    );
    println!(
        "  {} {:>10}",
        "Total Lines:".white(),
        format_number(summary.total_lines).cyan()
    );
    println!(
        "  {} {:>10}",
        "Code Lines:".white(),
        format_number(summary.code_lines).green()
    );
    println!(
        "  {} {:>10}",
        "Comments:".white(),
        format_number(summary.comment_lines).yellow()
    );
    println!(
        "  {} {:>10}",
        "Blank Lines:".white(),
        format_number(summary.blank_lines).dimmed()
    );
    println!();
}

/// Print language statistics table
fn print_language_table(languages: &[crate::types::LanguageStats]) {
    if languages.is_empty() {
        println!("{}", "No source files found.".yellow());
        return;
    }

    println!("{}", "📈 By Language".bold().white());
    println!("{}", "─".repeat(90).dimmed());

    // Header
    println!(
        "{:<15} {:>8} {:>10} {:>10} {:>10} {:>10} {:>8}",
        "Language".bold(),
        "Files".bold(),
        "Code".bold(),
        "Comments".bold(),
        "Blanks".bold(),
        "Total".bold(),
        "%".bold()
    );
    println!("{}", "─".repeat(90).dimmed());

    // Data rows
    for lang in languages {
        let color = get_language_color(&lang.language);
        println!(
            "{:<15} {:>8} {:>10} {:>10} {:>10} {:>10} {:>7.1}%",
            lang.language.color(color),
            format_number(lang.files),
            format_number(lang.code).green(),
            format_number(lang.comments).yellow(),
            format_number(lang.blanks).dimmed(),
            format_number(lang.lines),
            lang.percentage
        );
    }

    println!("{}", "─".repeat(90).dimmed());
    println!();
}

/// Print individual file details
fn print_file_details(files: &[crate::types::FileStats]) {
    println!("{}", "📁 By File".bold().white());
    println!("{}", "─".repeat(100).dimmed());

    // Header
    println!(
        "{:<50} {:<12} {:>8} {:>8} {:>8} {:>8}",
        "File".bold(),
        "Language".bold(),
        "Code".bold(),
        "Comments".bold(),
        "Blanks".bold(),
        "Total".bold()
    );
    println!("{}", "─".repeat(100).dimmed());

    for file in files {
        let path_str = file.path.to_string_lossy();
        let display_path = if path_str.len() > 48 {
            format!("...{}", &path_str[path_str.len() - 45..])
        } else {
            path_str.to_string()
        };

        let color = get_language_color(&file.language);
        println!(
            "{:<50} {:<12} {:>8} {:>8} {:>8} {:>8}",
            display_path,
            file.language.color(color),
            format_number(file.code).green(),
            format_number(file.comments).yellow(),
            format_number(file.blanks).dimmed(),
            format_number(file.lines)
        );
    }

    println!("{}", "─".repeat(100).dimmed());
    println!();
}

/// Format a number with thousands separator
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().rev().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    result.chars().rev().collect()
}

/// Get a color for a language (for visual differentiation)
fn get_language_color(language: &str) -> Color {
    match language {
        "Rust" => Color::TrueColor { r: 222, g: 165, b: 132 }, // Orange
        "TypeScript" | "JavaScript" => Color::TrueColor { r: 49, g: 120, b: 198 }, // Blue
        "Python" => Color::TrueColor { r: 55, g: 118, b: 171 }, // Python blue
        "Go" => Color::TrueColor { r: 0, g: 173, b: 216 }, // Go cyan
        "Java" => Color::TrueColor { r: 176, g: 114, b: 25 }, // Java orange
        "C" | "C++" => Color::TrueColor { r: 85, g: 85, b: 85 }, // Gray
        "C#" => Color::TrueColor { r: 104, g: 33, b: 122 }, // Purple
        "HTML" => Color::TrueColor { r: 227, g: 76, b: 38 }, // Orange-red
        "CSS" | "SCSS" => Color::TrueColor { r: 86, g: 61, b: 124 }, // Purple
        "JSON" | "YAML" | "TOML" => Color::TrueColor { r: 128, g: 128, b: 128 }, // Gray
        "Markdown" => Color::TrueColor { r: 8, g: 63, b: 136 }, // Dark blue
        "Shell" => Color::TrueColor { r: 0, g: 128, b: 0 }, // Green
        "Ruby" => Color::TrueColor { r: 204, g: 52, b: 45 }, // Red
        "PHP" => Color::TrueColor { r: 119, g: 123, b: 180 }, // Purple-ish
        "Swift" => Color::TrueColor { r: 240, g: 81, b: 56 }, // Orange
        "Kotlin" => Color::TrueColor { r: 169, g: 123, b: 255 }, // Purple
        "SQL" => Color::TrueColor { r: 0, g: 117, b: 143 }, // Teal
        "Vue" => Color::TrueColor { r: 65, g: 184, b: 131 }, // Green
        "Svelte" => Color::TrueColor { r: 255, g: 62, b: 0 }, // Orange
        _ => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number_small() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(1), "1");
        assert_eq!(format_number(123), "123");
    }

    #[test]
    fn test_format_number_thousands() {
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(12345), "12,345");
        assert_eq!(format_number(123456), "123,456");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_get_language_color() {
        // Just ensure these don't panic
        let _ = get_language_color("Rust");
        let _ = get_language_color("TypeScript");
        let _ = get_language_color("Python");
        let _ = get_language_color("Unknown");
    }
}
