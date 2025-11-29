/**
 * Port Scanner - Main Entry Point
 *
 * DEPENDENCY MAP:
 *
 * Dependencies:
 *   ├─ src/cli.rs
 *   ├─ src/scanner.rs
 *   ├─ src/types.rs
 *   └─ src/output.rs
 * Related Documentation:
 *   ├─ Spec: ./port_scanner.spec.md
 *   └─ Issue: docs/01_issues/open/2025_11/20251129_port_scanner.md
 */

mod cli;
mod output;
mod scanner;
mod types;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use output::{print_result, OutputFormat};
use scanner::scan_ports;
use types::Protocol;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // ポートをスキャン
    let mut result = scan_ports()?;

    // フィルタリング適用
    
    // プロトコルフィルタ
    match cli.protocol.to_lowercase().as_str() {
        "tcp" => {
            result = result.filter_by_protocol(Protocol::Tcp);
        }
        "udp" => {
            result = result.filter_by_protocol(Protocol::Udp);
        }
        _ => {} // "both" の場合はフィルタしない
    }

    // LISTENINGのみフィルタ
    if cli.listening {
        result = result.filter_listening_only();
    }

    // 開発ポートのみフィルタ
    if cli.dev_ports {
        result = result.filter_dev_ports_only();
    }

    // 特定ポートフィルタ
    if let Some(ref ports) = cli.port {
        result = result.filter_by_ports(ports);
    }

    // ポート範囲フィルタ
    if let Some((start, end)) = cli.parse_range() {
        result = result.filter_by_range(start, end);
    }

    // 結果を出力
    let format = OutputFormat::from_str(&cli.output);
    print_result(&result, format);

    Ok(())
}
