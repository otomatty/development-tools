/**
 * Port Scanner - CLI
 *
 * DEPENDENCY MAP:
 *
 * Parents (Files that import this module):
 *   └─ src/main.rs
 * Related Documentation:
 *   ├─ Spec: ./port_scanner.spec.md
 *   └─ Issue: docs/01_issues/open/2025_11/20251129_port_scanner.md
 */

use clap::Parser;

/// CLI引数の定義
#[derive(Parser, Debug)]
#[command(name = "port-scanner")]
#[command(about = "Scan local ports and show which processes are using them")]
#[command(version = "1.0.0")]
pub struct Cli {
    /// 特定ポートをチェック（カンマ区切りで複数指定可）
    #[arg(short, long, value_delimiter = ',')]
    pub port: Option<Vec<u16>>,

    /// ポート範囲（例: 3000-4000）
    #[arg(short, long)]
    pub range: Option<String>,

    /// プロトコル（tcp/udp/both）
    #[arg(long, default_value = "both")]
    pub protocol: String,

    /// 出力形式（text/json）
    #[arg(short, long, default_value = "text")]
    pub output: String,

    /// 開発用ポート（3000,5173,8080,8000,4200等）のみ表示
    #[arg(long)]
    pub dev_ports: bool,

    /// LISTENINGポートのみ表示
    #[arg(long)]
    pub listening: bool,
}

impl Cli {
    /// ポート範囲をパース
    /// start > end の場合は自動的にスワップする
    pub fn parse_range(&self) -> Option<(u16, u16)> {
        self.range.as_ref().and_then(|r| {
            let parts: Vec<&str> = r.split('-').collect();
            if parts.len() == 2 {
                let start = parts[0].trim().parse::<u16>().ok()?;
                let end = parts[1].trim().parse::<u16>().ok()?;
                // start > end の場合はスワップ
                if start > end {
                    Some((end, start))
                } else {
                    Some((start, end))
                }
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_range_valid() {
        let cli = Cli {
            port: None,
            range: Some("3000-4000".to_string()),
            protocol: "both".to_string(),
            output: "text".to_string(),
            dev_ports: false,
            listening: false,
        };

        let range = cli.parse_range();
        assert_eq!(range, Some((3000, 4000)));
    }

    #[test]
    fn test_parse_range_with_spaces() {
        let cli = Cli {
            port: None,
            range: Some("3000 - 4000".to_string()),
            protocol: "both".to_string(),
            output: "text".to_string(),
            dev_ports: false,
            listening: false,
        };

        let range = cli.parse_range();
        assert_eq!(range, Some((3000, 4000)));
    }

    #[test]
    fn test_parse_range_none() {
        let cli = Cli {
            port: None,
            range: None,
            protocol: "both".to_string(),
            output: "text".to_string(),
            dev_ports: false,
            listening: false,
        };

        let range = cli.parse_range();
        assert_eq!(range, None);
    }

    #[test]
    fn test_parse_range_invalid() {
        let cli = Cli {
            port: None,
            range: Some("invalid".to_string()),
            protocol: "both".to_string(),
            output: "text".to_string(),
            dev_ports: false,
            listening: false,
        };

        let range = cli.parse_range();
        assert_eq!(range, None);
    }

    // TC: parse_range - 逆順範囲（start > end）のスワップ
    #[test]
    fn test_parse_range_reversed() {
        let cli = Cli {
            port: None,
            range: Some("5000-3000".to_string()),
            protocol: "both".to_string(),
            output: "text".to_string(),
            dev_ports: false,
            listening: false,
        };

        let range = cli.parse_range();
        // start > end の場合は自動的にスワップされる
        assert_eq!(range, Some((3000, 5000)));
    }
}
