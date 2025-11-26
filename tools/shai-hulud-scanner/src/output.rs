//! Output formatting for scan results (text and JSON)

use colored::*;

use crate::cli::Args;
use crate::types::{Detection, PackageSource, Severity, SuspiciousFile};

/// Output results based on format
pub fn output_results(detections: &[Detection], suspicious: &[SuspiciousFile], args: &Args) {
    match args.output.as_str() {
        "json" => output_json(detections, suspicious),
        _ => output_text(detections, suspicious),
    }
}

/// Output results as text
fn output_text(detections: &[Detection], suspicious: &[SuspiciousFile]) {
    // Suspicious files first
    if !suspicious.is_empty() {
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════════".red()
        );
        println!(
            "{} {} suspicious files detected!",
            "⚠".red().bold(),
            suspicious.len()
        );
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════════".red()
        );
        println!();
        println!("{}", "SUSPICIOUS FILES:".red().bold());
        println!("{}", "─────────────────".red());

        for file in suspicious {
            println!(
                "\n{} {}",
                "[SUSPICIOUS]".red().bold(),
                file.path.display()
            );
            println!("   Reason: {}", file.reason.yellow());
        }
        println!();
    }

    if detections.is_empty() && suspicious.is_empty() {
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════════".green()
        );
        println!(
            "{} No affected packages or suspicious files detected!",
            "✓".green().bold()
        );
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════════".green()
        );
        return;
    }

    if !detections.is_empty() {
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════════".red()
        );
        println!(
            "{} {} package issues detected!",
            "⚠".red().bold(),
            detections.len()
        );
        println!(
            "{}",
            "═══════════════════════════════════════════════════════════════".red()
        );
        println!();

        let critical_count = detections
            .iter()
            .filter(|d| matches!(d.severity, Severity::Critical))
            .count();
        let warning_count = detections
            .iter()
            .filter(|d| matches!(d.severity, Severity::Warning))
            .count();

        if critical_count > 0 {
            println!("{}", "CRITICAL FINDINGS (exact version match):".red().bold());
            println!("{}", "─────────────────────────────────────────".red());
            for detection in detections
                .iter()
                .filter(|d| matches!(d.severity, Severity::Critical))
            {
                print_detection(detection);
            }
            println!();
        }

        if warning_count > 0 {
            println!(
                "{}",
                "WARNINGS (package name match, different version):"
                    .yellow()
                    .bold()
            );
            println!(
                "{}",
                "─────────────────────────────────────────────────".yellow()
            );
            for detection in detections
                .iter()
                .filter(|d| matches!(d.severity, Severity::Warning))
            {
                print_detection(detection);
            }
        }
    }
}

/// Print a single detection
fn print_detection(detection: &Detection) {
    let severity_str = match detection.severity {
        Severity::Critical => format!("[{}]", "CRITICAL".red().bold()),
        Severity::Warning => format!("[{}]", "WARNING".yellow().bold()),
    };

    let source_str = format!("({})", detection.package.source);

    println!(
        "\n{} {} @ {} {}",
        severity_str,
        detection.package.name.bold(),
        detection.package.version.cyan(),
        source_str.dimmed()
    );
    println!("   Location: {}", detection.package.location.display());
    println!("   File: {}", detection.package.file_type);
    println!(
        "   Affected versions: {}",
        detection.affected_versions.join(", ").red()
    );

    if matches!(detection.severity, Severity::Critical) {
        println!(
            "   {} This exact version is known to be compromised!",
            "⚠".red()
        );
    }
}

/// Output results as JSON
fn output_json(detections: &[Detection], suspicious: &[SuspiciousFile]) {
    let json_output = serde_json::json!({
        "detections": detections.iter().map(|d| {
            serde_json::json!({
                "package": d.package.name,
                "installed_version": d.package.version,
                "location": d.package.location.to_string_lossy(),
                "file_type": d.package.file_type,
                "source": format!("{}", d.package.source),
                "affected_versions": d.affected_versions,
                "severity": format!("{}", d.severity),
            })
        }).collect::<Vec<_>>(),
        "suspicious_files": suspicious.iter().map(|s| {
            serde_json::json!({
                "path": s.path.to_string_lossy(),
                "reason": s.reason,
                "severity": format!("{}", s.severity),
            })
        }).collect::<Vec<_>>(),
    });

    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
}

/// Print summary
pub fn print_summary(detections: &[Detection], suspicious: &[SuspiciousFile]) {
    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════".cyan()
    );
    println!("{}", "                          SUMMARY".cyan().bold());
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════".cyan()
    );

    let critical_count = detections
        .iter()
        .filter(|d| matches!(d.severity, Severity::Critical))
        .count();
    let warning_count = detections
        .iter()
        .filter(|d| matches!(d.severity, Severity::Warning))
        .count();
    let suspicious_count = suspicious.len();

    // Count by source
    let global_count = detections
        .iter()
        .filter(|d| d.package.source != PackageSource::Local)
        .count();

    if suspicious_count > 0 {
        println!(
            "{} {} Suspicious files detected",
            "●".red(),
            suspicious_count
        );
    }
    if critical_count > 0 {
        println!(
            "{} {} Critical issues (exact version match)",
            "●".red(),
            critical_count
        );
    }
    if warning_count > 0 {
        println!(
            "{} {} Warnings (package name match)",
            "●".yellow(),
            warning_count
        );
    }
    if global_count > 0 {
        println!(
            "{} {} issues in global packages",
            "●".magenta(),
            global_count
        );
    }
    if detections.is_empty() && suspicious.is_empty() {
        println!(
            "{} No affected packages or suspicious files found",
            "●".green()
        );
    }

    println!();
    println!("{}", "Scanned:".bold());
    println!("  {} Local projects (package.json, lockfiles)", "→".blue());
    println!("  {} Global packages (npm, yarn, pnpm, bun)", "→".blue());
    println!("  {} Suspicious files and binaries", "→".blue());

    println!();
    println!("{}", "Recommended Actions:".bold());
    if critical_count > 0 || suspicious_count > 0 {
        println!(
            "  1. {} Immediately review and remove/update critical packages",
            "→".red()
        );
        println!(
            "  2. {} Rotate all secrets (API keys, tokens, SSH keys)",
            "→".red()
        );
        println!(
            "  3. {} Check GitHub for suspicious repositories",
            "→".red()
        );
        println!("  4. {} Review and remove suspicious files", "→".red());
    }
    if warning_count > 0 {
        println!(
            "  {} Consider reviewing warning packages for recent updates",
            "→".yellow()
        );
    }
    println!(
        "  {} Add ignore-scripts=true to .npmrc / bunfig.toml",
        "→".blue()
    );
    println!(
        "  {} Run `npm audit` or `bun audit` for additional checks",
        "→".blue()
    );
    println!();
    println!(
        "Reference: {}",
        "https://codebook.machinarecord.com/threatreport/silobreaker-cyber-alert/42718/".underline()
    );
}

