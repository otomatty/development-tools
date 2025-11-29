/**
 * Port Scanner - Output
 *
 * DEPENDENCY MAP:
 *
 * Parents (Files that import this module):
 *   └─ src/main.rs
 * Dependencies:
 *   └─ src/types.rs
 * Related Documentation:
 *   ├─ Spec: ./port_scanner.spec.md
 *   └─ Issue: docs/01_issues/open/2025_11/20251129_port_scanner.md
 */

use crate::types::{DevPortStatus, PortState, ScanResult, DEV_PORTS};
use colored::Colorize;

/// 出力フォーマット
pub enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            _ => OutputFormat::Text,
        }
    }
}

/// 結果を出力
pub fn print_result(result: &ScanResult, format: OutputFormat) {
    match format {
        OutputFormat::Json => print_json(result),
        OutputFormat::Text => print_text(result),
    }
}

/// JSON形式で出力
fn print_json(result: &ScanResult) {
    match serde_json::to_string_pretty(result) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

/// テキスト形式で出力
fn print_text(result: &ScanResult) {
    // ヘッダー
    println!("\n{}", "═".repeat(80).cyan());
    println!("{}", " 🔍 Port Scanner Results ".cyan().bold());
    println!("{}", "═".repeat(80).cyan());

    // サマリー
    println!("\n{}", "📊 Summary".bold());
    println!("  Total Ports:      {}", result.summary.total_ports.to_string().green());
    println!("  TCP Ports:        {}", result.summary.tcp_ports.to_string().blue());
    println!("  UDP Ports:        {}", result.summary.udp_ports.to_string().yellow());
    println!("  Dev Ports In Use: {}", result.summary.dev_ports_in_use.to_string().magenta());

    // 開発ポートステータス
    println!("\n{}", "🛠️  Dev Port Status".bold());
    for &port in DEV_PORTS.iter() {
        let status = result.dev_port_status.get(&port.to_string())
            .unwrap_or(&DevPortStatus::Available);
        let status_str = match status {
            DevPortStatus::InUse => "🔴 IN USE".red(),
            DevPortStatus::Available => "🟢 AVAILABLE".green(),
        };
        
        // 使用中の場合はプロセス名も表示
        let process = result.ports.iter()
            .find(|p| p.port == port)
            .and_then(|p| p.process_name.clone())
            .unwrap_or_default();
        
        if !process.is_empty() {
            println!("  {:5} {} ({})", port, status_str, process.dimmed());
        } else {
            println!("  {:5} {}", port, status_str);
        }
    }

    // ポート詳細
    if !result.ports.is_empty() {
        println!("\n{}", "📋 Port Details".bold());
        println!("  {:<6} {:<6} {:<12} {:<15} {:<8} {}",
            "PORT".bold(), "PROTO".bold(), "STATE".bold(), 
            "PROCESS".bold(), "PID".bold(), "ADDRESS".bold());
        println!("  {}", "-".repeat(70).dimmed());

        for port_info in &result.ports {
            let state_colored = match port_info.state {
                PortState::Listening => "LISTENING".green(),
                PortState::Established => "ESTABLISHED".yellow(),
                _ => port_info.state.to_string().normal(),
            };

            let process = port_info.process_name.as_deref().unwrap_or("-");
            let pid = port_info.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string());

            let port_str = if port_info.is_dev_port {
                port_info.port.to_string().magenta().bold()
            } else {
                port_info.port.to_string().normal()
            };

            println!("  {:<6} {:<6} {:<12} {:<15} {:<8} {}",
                port_str,
                port_info.protocol.to_string().blue(),
                state_colored,
                process,
                pid,
                port_info.local_address.dimmed());
        }
    } else {
        println!("\n{}", "No ports found.".dimmed());
    }

    println!("\n{}", "═".repeat(80).cyan());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PortInfo, Protocol, ScanResult};

    // TC-010: JSON出力形式
    #[test]
    fn test_json_output_format() {
        let ports = vec![
            PortInfo {
                port: 3000,
                protocol: Protocol::Tcp,
                state: PortState::Listening,
                pid: Some(12345),
                process_name: Some("node".to_string()),
                command: None,
                local_address: "127.0.0.1".to_string(),
                is_dev_port: true,
            },
        ];

        let result = ScanResult::new(ports);
        let json = serde_json::to_string_pretty(&result).unwrap();

        // JSONが正しい構造を持っているか確認
        assert!(json.contains("\"summary\""));
        assert!(json.contains("\"ports\""));
        assert!(json.contains("\"dev_port_status\""));
        assert!(json.contains("\"total_ports\""));
        assert!(json.contains("\"port\": 3000"));
        assert!(json.contains("\"process_name\": \"node\""));
    }

    #[test]
    fn test_output_format_from_str() {
        assert!(matches!(OutputFormat::from_str("json"), OutputFormat::Json));
        assert!(matches!(OutputFormat::from_str("JSON"), OutputFormat::Json));
        assert!(matches!(OutputFormat::from_str("text"), OutputFormat::Text));
        assert!(matches!(OutputFormat::from_str("TEXT"), OutputFormat::Text));
        assert!(matches!(OutputFormat::from_str("anything"), OutputFormat::Text));
    }

    #[test]
    fn test_json_structure() {
        let result = ScanResult::new(vec![]);
        let json = serde_json::to_string(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        // 必要なキーが存在することを確認
        assert!(parsed.get("summary").is_some());
        assert!(parsed.get("ports").is_some());
        assert!(parsed.get("dev_port_status").is_some());
        
        // summaryの構造を確認
        let summary = parsed.get("summary").unwrap();
        assert!(summary.get("total_ports").is_some());
        assert!(summary.get("tcp_ports").is_some());
        assert!(summary.get("udp_ports").is_some());
        assert!(summary.get("dev_ports_in_use").is_some());
    }
}
