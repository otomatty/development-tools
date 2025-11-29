/**
 * Port Scanner - Scanner
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

use crate::types::{PortInfo, PortState, Protocol, ScanResult};
use anyhow::{Context, Result};
use regex::Regex;
use std::process::Command;

/// OSを検出して適切なスキャナーを選択
pub fn scan_ports() -> Result<ScanResult> {
    #[cfg(target_os = "macos")]
    {
        scan_ports_macos()
    }

    #[cfg(target_os = "linux")]
    {
        scan_ports_linux()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err(anyhow::anyhow!("Unsupported operating system"))
    }
}

/// macOS用: lsofコマンドを使用してポートをスキャン
#[cfg(target_os = "macos")]
fn scan_ports_macos() -> Result<ScanResult> {
    let output = Command::new("lsof")
        .args(["-i", "-P", "-n"])
        .output()
        .context("Failed to execute lsof command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("lsof command failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ports = parse_lsof_output(&stdout);
    
    Ok(ScanResult::new(ports))
}

/// Linux用: ssコマンドを使用してポートをスキャン
#[cfg(target_os = "linux")]
fn scan_ports_linux() -> Result<ScanResult> {
    let output = Command::new("ss")
        .args(["-tulnp"])
        .output()
        .context("Failed to execute ss command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("ss command failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ports = parse_ss_output(&stdout);
    
    Ok(ScanResult::new(ports))
}

/// lsofの出力をパース（macOS用）
/// 
/// lsof -i -P -n の出力例:
/// COMMAND     PID   USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
/// node      12345   user   23u  IPv4 0x...           0t0  TCP 127.0.0.1:3000 (LISTEN)
/// chrome    67890   user   50u  IPv4 0x...           0t0  TCP 192.168.1.1:52345->142.250.196.46:443 (ESTABLISHED)
pub fn parse_lsof_output(output: &str) -> Vec<PortInfo> {
    let mut ports = Vec::new();
    
    for line in output.lines().skip(1) { // ヘッダー行をスキップ
        if let Some(port_info) = parse_lsof_line(line) {
            // 重複を除去（同じポート・プロトコルの組み合わせ）
            if !ports.iter().any(|p: &PortInfo| p.port == port_info.port && p.protocol == port_info.protocol && p.state == port_info.state) {
                ports.push(port_info);
            }
        }
    }
    
    ports
}

/// lsofの1行をパース
pub fn parse_lsof_line(line: &str) -> Option<PortInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    // 最低限必要な要素数（UDPは状態がないので9、TCPは10）
    if parts.len() < 9 {
        return None;
    }

    let process_name = parts[0].to_string();
    let pid: u32 = parts[1].parse().ok()?;
    
    // TCPまたはUDPを含む行のみ処理
    let protocol = if line.contains("TCP") {
        Protocol::Tcp
    } else if line.contains("UDP") {
        Protocol::Udp
    } else {
        return None;
    };

    // 状態が括弧で囲まれている場合を探す
    // 例: "(LISTEN)" "(ESTABLISHED)"
    let state_part = parts.iter()
        .find(|p| p.starts_with('(') && p.ends_with(')'))
        .map(|s| s.trim_matches(|c| c == '(' || c == ')'));

    // アドレス:ポートを抽出
    // "127.0.0.1:3000" または "192.168.1.1:52345->142.250.196.46:443" または "*:5353"
    let name_str = parts.iter()
        .find(|p| p.contains(':') && !p.starts_with('('))?;
    
    let (local_address, port) = parse_address_port(name_str)?;
    
    let state = state_part
        .map(PortState::from_str)
        .unwrap_or(PortState::Unknown);

    let mut port_info = PortInfo::new(port, protocol, state, local_address);
    port_info.pid = Some(pid);
    port_info.process_name = Some(process_name);
    
    Some(port_info)
}

/// アドレス:ポート文字列をパース
/// "127.0.0.1:3000" -> ("127.0.0.1", 3000)
/// "[::1]:3000" -> ("::1", 3000)
/// "192.168.1.1:52345->142.250.196.46:443" -> ("192.168.1.1", 52345)
fn parse_address_port(s: &str) -> Option<(String, u16)> {
    // 接続先を含む場合は矢印の前の部分を取得
    let local_part = s.split("->").next()?;
    
    // IPv6アドレスの場合 [::1]:3000
    if local_part.starts_with('[') {
        let re = Regex::new(r"\[([^\]]+)\]:(\d+)").ok()?;
        let caps = re.captures(local_part)?;
        let addr = caps.get(1)?.as_str().to_string();
        let port: u16 = caps.get(2)?.as_str().parse().ok()?;
        return Some((addr, port));
    }
    
    // IPv4アドレスの場合 127.0.0.1:3000
    let parts: Vec<&str> = local_part.rsplitn(2, ':').collect();
    if parts.len() == 2 {
        let port: u16 = parts[0].parse().ok()?;
        let addr = parts[1].to_string();
        return Some((addr, port));
    }
    
    None
}

/// ssの出力をパース（Linux用）
///
/// ss -tulnp の出力例:
/// Netid State  Recv-Q Send-Q Local Address:Port Peer Address:Port Process
/// tcp   LISTEN 0      128    127.0.0.1:3000     0.0.0.0:*          users:(("node",pid=12345,fd=23))
#[cfg(target_os = "linux")]
pub fn parse_ss_output(output: &str) -> Vec<PortInfo> {
    let mut ports = Vec::new();
    
    for line in output.lines().skip(1) { // ヘッダー行をスキップ
        if let Some(port_info) = parse_ss_line(line) {
            if !ports.iter().any(|p: &PortInfo| p.port == port_info.port && p.protocol == port_info.protocol) {
                ports.push(port_info);
            }
        }
    }
    
    ports
}

/// ssの1行をパース
#[cfg(target_os = "linux")]
pub fn parse_ss_line(line: &str) -> Option<PortInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 5 {
        return None;
    }

    // プロトコル
    let protocol = match parts[0] {
        "tcp" => Protocol::Tcp,
        "udp" => Protocol::Udp,
        _ => return None,
    };

    // 状態
    let state = PortState::from_str(parts[1]);

    // ローカルアドレス:ポート
    let local_addr_port = parts[4];
    let (local_address, port) = parse_address_port(local_addr_port)?;

    let mut port_info = PortInfo::new(port, protocol, state, local_address);

    // プロセス情報を抽出（最後のフィールドに含まれる可能性）
    // users:(("node",pid=12345,fd=23))
    if let Some(process_part) = parts.iter().find(|p| p.starts_with("users:")) {
        let re = Regex::new(r#"\("([^"]+)",pid=(\d+)"#).ok()?;
        if let Some(caps) = re.captures(process_part) {
            port_info.process_name = caps.get(1).map(|m| m.as_str().to_string());
            port_info.pid = caps.get(2).and_then(|m| m.as_str().parse().ok());
        }
    }

    Some(port_info)
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-002: parse_lsof_line - 正常なLISTENING行
    #[test]
    fn test_parse_lsof_line_listening() {
        let line = "node      12345   user   23u  IPv4 0x12345      0t0  TCP 127.0.0.1:3000 (LISTEN)";
        let result = parse_lsof_line(line);
        
        assert!(result.is_some());
        let port_info = result.unwrap();
        assert_eq!(port_info.port, 3000);
        assert_eq!(port_info.protocol, Protocol::Tcp);
        assert_eq!(port_info.state, PortState::Listening);
        assert_eq!(port_info.pid, Some(12345));
        assert_eq!(port_info.process_name, Some("node".to_string()));
        assert_eq!(port_info.local_address, "127.0.0.1");
    }

    // TC-003: parse_lsof_line - ESTABLISHED行
    #[test]
    fn test_parse_lsof_line_established() {
        let line = "chrome    67890   user   50u  IPv4 0x12345      0t0  TCP 192.168.1.1:52345->142.250.196.46:443 (ESTABLISHED)";
        let result = parse_lsof_line(line);
        
        assert!(result.is_some());
        let port_info = result.unwrap();
        assert_eq!(port_info.port, 52345);
        assert_eq!(port_info.protocol, Protocol::Tcp);
        assert_eq!(port_info.state, PortState::Established);
        assert_eq!(port_info.local_address, "192.168.1.1");
    }

    #[test]
    fn test_parse_lsof_line_udp() {
        let line = "mDNSRespo  1234   root   12u  IPv4 0x12345      0t0  UDP *:5353";
        let result = parse_lsof_line(line);
        
        assert!(result.is_some());
        let port_info = result.unwrap();
        assert_eq!(port_info.port, 5353);
        assert_eq!(port_info.protocol, Protocol::Udp);
    }

    #[test]
    fn test_parse_lsof_line_ipv6() {
        let line = "node      12345   user   23u  IPv6 0x12345      0t0  TCP [::1]:3000 (LISTEN)";
        let result = parse_lsof_line(line);
        
        assert!(result.is_some());
        let port_info = result.unwrap();
        assert_eq!(port_info.port, 3000);
        assert_eq!(port_info.local_address, "::1");
    }

    #[test]
    fn test_parse_lsof_line_invalid() {
        let line = "invalid line";
        let result = parse_lsof_line(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_lsof_output_multiple_lines() {
        let output = r#"COMMAND     PID   USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node      12345   user   23u  IPv4 0x12345      0t0  TCP 127.0.0.1:3000 (LISTEN)
node      12345   user   24u  IPv4 0x12346      0t0  TCP 127.0.0.1:3000 (LISTEN)
vite      67890   user   25u  IPv4 0x12347      0t0  TCP 127.0.0.1:5173 (LISTEN)"#;

        let ports = parse_lsof_output(output);
        
        // 重複は除去される
        assert_eq!(ports.len(), 2);
        assert!(ports.iter().any(|p| p.port == 3000));
        assert!(ports.iter().any(|p| p.port == 5173));
    }

    #[test]
    fn test_parse_address_port_ipv4() {
        let result = parse_address_port("127.0.0.1:3000");
        assert_eq!(result, Some(("127.0.0.1".to_string(), 3000)));
    }

    #[test]
    fn test_parse_address_port_ipv6() {
        let result = parse_address_port("[::1]:3000");
        assert_eq!(result, Some(("::1".to_string(), 3000)));
    }

    #[test]
    fn test_parse_address_port_with_remote() {
        let result = parse_address_port("192.168.1.1:52345->142.250.196.46:443");
        assert_eq!(result, Some(("192.168.1.1".to_string(), 52345)));
    }

    #[test]
    fn test_parse_address_port_wildcard() {
        let result = parse_address_port("*:5353");
        assert_eq!(result, Some(("*".to_string(), 5353)));
    }

    // TC-004: parse_ss_line - Linux ss出力
    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_ss_line_listening() {
        let line = r#"tcp   LISTEN 0      128    127.0.0.1:3000     0.0.0.0:*          users:(("node",pid=12345,fd=23))"#;
        let result = parse_ss_line(line);
        
        assert!(result.is_some());
        let port_info = result.unwrap();
        assert_eq!(port_info.port, 3000);
        assert_eq!(port_info.protocol, Protocol::Tcp);
        assert_eq!(port_info.state, PortState::Listening);
        assert_eq!(port_info.pid, Some(12345));
        assert_eq!(port_info.process_name, Some("node".to_string()));
    }
}
