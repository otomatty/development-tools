//! Command line argument definitions

use clap::Parser;
use std::path::PathBuf;

/// Default URL for the Shai-Hulud affected packages CSV
pub const DEFAULT_CSV_URL: &str =
    "https://raw.githubusercontent.com/wiz-sec-public/wiz-research-iocs/main/reports/shai-hulud-2-packages.csv";

/// Command line arguments
#[derive(Parser, Debug)]
#[command(name = "shai-hulud-scanner")]
#[command(about = "Scan for npm packages affected by the Shai-Hulud supply chain attack")]
#[command(version)]
pub struct Args {
    /// Directory to scan (defaults to home directory for full PC scan)
    #[arg(short, long)]
    pub scan_dir: Option<PathBuf>,

    /// Scan current directory only (instead of home directory)
    #[arg(long)]
    pub current_dir: bool,

    /// Path to local CSV file (if not provided, downloads from GitHub)
    #[arg(short, long)]
    pub csv_file: Option<PathBuf>,

    /// URL to download CSV from (defaults to Wiz Security's GitHub)
    #[arg(long, default_value = DEFAULT_CSV_URL)]
    pub csv_url: String,

    /// Output format: text, json
    #[arg(short, long, default_value = "text")]
    pub output: String,

    /// Show all scanned packages (not just affected ones)
    #[arg(long)]
    pub verbose: bool,

    /// Skip downloading CSV and use cached version if available
    #[arg(long)]
    pub offline: bool,

    /// Skip global packages scan
    #[arg(long)]
    pub skip_global: bool,

    /// Skip suspicious file detection
    #[arg(long)]
    pub skip_suspicious: bool,
}

