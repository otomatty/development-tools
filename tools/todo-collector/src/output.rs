//! Output formatting for TODO Collector

use colored::*;

use crate::cli::OutputFormat;
use crate::types::{ScanResult, TodoType};

/// Output the scan results in the specified format
pub fn output_results(result: &ScanResult, format: OutputFormat) {
    match format {
        OutputFormat::Json => output_json(result),
        OutputFormat::Text => output_text(result),
    }
}

/// Output results as JSON
fn output_json(result: &ScanResult) {
    match serde_json::to_string_pretty(result) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

/// Output results as colored text
fn output_text(result: &ScanResult) {
    // Print header
    println!();
    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());
    println!("{}", "                    TODO Collector Results                      ".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());
    println!();

    // Print summary
    print_summary(result);
    println!();

    if result.items.is_empty() {
        println!("{}", "  ✓ No TODO comments found!".green());
        return;
    }

    // Print items grouped by file
    let mut current_file = String::new();
    for item in &result.items {
        if item.file != current_file {
            if !current_file.is_empty() {
                println!();
            }
            println!("{}", format!("📁 {}", item.file).white().bold());
            current_file = item.file.clone();
        }

        let type_colored = colorize_type(&item.todo_type);
        let line_str = format!("L{:>4}", item.line).dimmed();
        println!("  {} {} {}", line_str, type_colored, item.content);
    }
    println!();
}

/// Print summary statistics
fn print_summary(result: &ScanResult) {
    println!("  {} Total: {}", "📊".to_string(), result.summary.total.to_string().bold());
    println!();

    // Order by priority
    let type_order = [
        ("FIXME", TodoType::Fixme),
        ("TODO", TodoType::Todo),
        ("HACK", TodoType::Hack),
        ("XXX", TodoType::Xxx),
        ("NOTE", TodoType::Note),
    ];

    for (name, todo_type) in type_order {
        if let Some(&count) = result.summary.by_type.get(name) {
            if count > 0 {
                let colored_name = colorize_type(&todo_type);
                let bar = "█".repeat(count.min(20));
                let bar_colored = match todo_type {
                    TodoType::Fixme => bar.red(),
                    TodoType::Todo => bar.blue(),
                    TodoType::Hack => bar.yellow(),
                    TodoType::Xxx => bar.magenta(),
                    TodoType::Note => bar.green(),
                };
                println!("  {:>8}: {:>4} {}", colored_name, count, bar_colored);
            }
        }
    }
}

/// Colorize a TODO type for terminal output
fn colorize_type(todo_type: &TodoType) -> ColoredString {
    match todo_type {
        TodoType::Fixme => "FIXME".red().bold(),
        TodoType::Todo => "TODO ".blue().bold(),
        TodoType::Hack => "HACK ".yellow().bold(),
        TodoType::Xxx => "XXX  ".magenta().bold(),
        TodoType::Note => "NOTE ".green().bold(),
    }
}

