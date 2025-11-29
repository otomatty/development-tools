/**
 * Port Scanner - Types
 *
 * DEPENDENCY MAP:
 *
 * Parents (Files that import this module):
 *   ├─ src/scanner.rs
 *   ├─ src/output.rs
 *   └─ src/main.rs
 * Related Documentation:
 *   ├─ Spec: ./port_scanner.spec.md
 *   └─ Issue: docs/01_issues/open/2025_11/20251129_port_scanner.md
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// 開発でよく使われるポート一覧（ソート済み - binary_search用）
pub const DEV_PORTS: &[u16] = &[
    1420,  // Tauri
    3000,  // React, Express, Rails
    3001,  // Next.js (dev alt)
    3306,  // MySQL
    4200,  // Angular
    4321,  // Astro
    5000,  // Flask, ASP.NET
    5173,  // Vite
    5432,  // PostgreSQL
    6379,  // Redis
    8000,  // Django, PHP
    8080,  // Tomcat, 汎用
    8888,  // Jupyter
    27017, // MongoDB
];

/// プロトコル種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
        }
    }
}

/// ポートの状態
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PortState {
    Listening,
    Established,
    TimeWait,
    CloseWait,
    SynSent,
    SynReceived,
    FinWait1,
    FinWait2,
    Closing,
    LastAck,
    Unknown,
}

impl fmt::Display for PortState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortState::Listening => write!(f, "LISTENING"),
            PortState::Established => write!(f, "ESTABLISHED"),
            PortState::TimeWait => write!(f, "TIME_WAIT"),
            PortState::CloseWait => write!(f, "CLOSE_WAIT"),
            PortState::SynSent => write!(f, "SYN_SENT"),
            PortState::SynReceived => write!(f, "SYN_RECEIVED"),
            PortState::FinWait1 => write!(f, "FIN_WAIT_1"),
            PortState::FinWait2 => write!(f, "FIN_WAIT_2"),
            PortState::Closing => write!(f, "CLOSING"),
            PortState::LastAck => write!(f, "LAST_ACK"),
            PortState::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl PortState {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "LISTEN" | "LISTENING" => PortState::Listening,
            "ESTABLISHED" | "ESTAB" => PortState::Established,
            "TIME_WAIT" | "TIME-WAIT" => PortState::TimeWait,
            "CLOSE_WAIT" | "CLOSE-WAIT" => PortState::CloseWait,
            "SYN_SENT" | "SYN-SENT" => PortState::SynSent,
            "SYN_RECV" | "SYN_RECEIVED" | "SYN-RECV" => PortState::SynReceived,
            "FIN_WAIT_1" | "FIN-WAIT-1" | "FIN_WAIT1" => PortState::FinWait1,
            "FIN_WAIT_2" | "FIN-WAIT-2" | "FIN_WAIT2" => PortState::FinWait2,
            "CLOSING" => PortState::Closing,
            "LAST_ACK" | "LAST-ACK" => PortState::LastAck,
            _ => PortState::Unknown,
        }
    }
}

/// 開発ポートの状態
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DevPortStatus {
    InUse,
    Available,
}

impl fmt::Display for DevPortStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DevPortStatus::InUse => write!(f, "in_use"),
            DevPortStatus::Available => write!(f, "available"),
        }
    }
}

/// 個別のポート情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub port: u16,
    pub protocol: Protocol,
    pub state: PortState,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub command: Option<String>,
    pub local_address: String,
    pub is_dev_port: bool,
}

impl PortInfo {
    pub fn new(port: u16, protocol: Protocol, state: PortState, local_address: String) -> Self {
        Self {
            port,
            protocol,
            state,
            pid: None,
            process_name: None,
            command: None,
            local_address,
            is_dev_port: is_dev_port(port),
        }
    }
}

/// スキャン結果のサマリー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub total_ports: usize,
    pub tcp_ports: usize,
    pub udp_ports: usize,
    pub dev_ports_in_use: usize,
}

impl Default for ScanSummary {
    fn default() -> Self {
        Self {
            total_ports: 0,
            tcp_ports: 0,
            udp_ports: 0,
            dev_ports_in_use: 0,
        }
    }
}

/// スキャン結果全体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub summary: ScanSummary,
    pub ports: Vec<PortInfo>,
    pub dev_port_status: HashMap<String, DevPortStatus>,
}

impl ScanResult {
    pub fn new(ports: Vec<PortInfo>) -> Self {
        let mut result = Self {
            summary: ScanSummary::default(),
            ports: Vec::new(),
            dev_port_status: HashMap::new(),
        };
        result.set_ports(ports);
        result
    }

    pub fn set_ports(&mut self, ports: Vec<PortInfo>) {
        self.ports = ports;
        self.recalculate_summary();
        self.recalculate_dev_port_status();
    }

    fn recalculate_summary(&mut self) {
        self.summary.total_ports = self.ports.len();
        self.summary.tcp_ports = self.ports.iter().filter(|p| p.protocol == Protocol::Tcp).count();
        self.summary.udp_ports = self.ports.iter().filter(|p| p.protocol == Protocol::Udp).count();
        self.summary.dev_ports_in_use = self.ports.iter().filter(|p| p.is_dev_port).count();
    }

    fn recalculate_dev_port_status(&mut self) {
        // 全ての開発ポートをavailableで初期化
        for &port in DEV_PORTS {
            self.dev_port_status.insert(port.to_string(), DevPortStatus::Available);
        }
        
        // 使用中のポートをin_useに更新
        for port_info in &self.ports {
            if port_info.is_dev_port {
                self.dev_port_status.insert(port_info.port.to_string(), DevPortStatus::InUse);
            }
        }
    }

    /// プロトコルでフィルタリング
    pub fn filter_by_protocol(mut self, protocol: Protocol) -> Self {
        self.ports.retain(|p| p.protocol == protocol);
        self.recalculate_summary();
        self.recalculate_dev_port_status();
        self
    }

    /// LISTENINGのみフィルタリング
    pub fn filter_listening_only(mut self) -> Self {
        self.ports.retain(|p| p.state == PortState::Listening);
        self.recalculate_summary();
        self.recalculate_dev_port_status();
        self
    }

    /// 開発ポートのみフィルタリング
    pub fn filter_dev_ports_only(mut self) -> Self {
        self.ports.retain(|p| p.is_dev_port);
        self.recalculate_summary();
        self.recalculate_dev_port_status();
        self
    }

    /// 特定ポートでフィルタリング
    pub fn filter_by_ports(mut self, ports: &[u16]) -> Self {
        self.ports.retain(|p| ports.contains(&p.port));
        self.recalculate_summary();
        self.recalculate_dev_port_status();
        self
    }

    /// ポート範囲でフィルタリング
    pub fn filter_by_range(mut self, start: u16, end: u16) -> Self {
        self.ports.retain(|p| p.port >= start && p.port <= end);
        self.recalculate_summary();
        self.recalculate_dev_port_status();
        self
    }
}

/// 指定ポートが開発用ポートかどうか判定
/// DEV_PORTSはソート済みなのでbinary_searchを使用
pub fn is_dev_port(port: u16) -> bool {
    DEV_PORTS.binary_search(&port).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-005: is_dev_port - 開発ポート判定
    #[test]
    fn test_is_dev_port_true() {
        assert!(is_dev_port(3000));
        assert!(is_dev_port(5173));
        assert!(is_dev_port(8080));
    }

    // TC-006: is_dev_port - 非開発ポート判定
    #[test]
    fn test_is_dev_port_false() {
        assert!(!is_dev_port(22));
        assert!(!is_dev_port(443));
        assert!(!is_dev_port(12345));
    }

    #[test]
    fn test_port_state_from_str() {
        assert_eq!(PortState::from_str("LISTEN"), PortState::Listening);
        assert_eq!(PortState::from_str("LISTENING"), PortState::Listening);
        assert_eq!(PortState::from_str("ESTABLISHED"), PortState::Established);
        assert_eq!(PortState::from_str("ESTAB"), PortState::Established);
        assert_eq!(PortState::from_str("TIME_WAIT"), PortState::TimeWait);
        assert_eq!(PortState::from_str("UNKNOWN_STATE"), PortState::Unknown);
    }

    #[test]
    fn test_protocol_display() {
        assert_eq!(format!("{}", Protocol::Tcp), "TCP");
        assert_eq!(format!("{}", Protocol::Udp), "UDP");
    }

    // TC-001: scan_ports - 空の結果
    #[test]
    fn test_scan_result_empty() {
        let result = ScanResult::new(vec![]);
        assert_eq!(result.summary.total_ports, 0);
        assert_eq!(result.summary.tcp_ports, 0);
        assert_eq!(result.summary.udp_ports, 0);
        assert_eq!(result.summary.dev_ports_in_use, 0);
        assert!(result.ports.is_empty());
    }

    #[test]
    fn test_scan_result_with_ports() {
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
            PortInfo {
                port: 8080,
                protocol: Protocol::Tcp,
                state: PortState::Listening,
                pid: Some(67890),
                process_name: Some("java".to_string()),
                command: None,
                local_address: "0.0.0.0".to_string(),
                is_dev_port: true,
            },
        ];

        let result = ScanResult::new(ports);
        assert_eq!(result.summary.total_ports, 2);
        assert_eq!(result.summary.tcp_ports, 2);
        assert_eq!(result.summary.udp_ports, 0);
        assert_eq!(result.summary.dev_ports_in_use, 2);
    }

    // TC-007: filter_by_protocol - TCPのみフィルタ
    #[test]
    fn test_filter_by_protocol() {
        let ports = vec![
            PortInfo {
                port: 3000,
                protocol: Protocol::Tcp,
                state: PortState::Listening,
                pid: None,
                process_name: None,
                command: None,
                local_address: "127.0.0.1".to_string(),
                is_dev_port: true,
            },
            PortInfo {
                port: 53,
                protocol: Protocol::Udp,
                state: PortState::Unknown,
                pid: None,
                process_name: None,
                command: None,
                local_address: "0.0.0.0".to_string(),
                is_dev_port: false,
            },
        ];

        let result = ScanResult::new(ports).filter_by_protocol(Protocol::Tcp);
        assert_eq!(result.summary.total_ports, 1);
        assert_eq!(result.summary.tcp_ports, 1);
        assert_eq!(result.summary.udp_ports, 0);
    }

    #[test]
    fn test_filter_listening_only() {
        let ports = vec![
            PortInfo {
                port: 3000,
                protocol: Protocol::Tcp,
                state: PortState::Listening,
                pid: None,
                process_name: None,
                command: None,
                local_address: "127.0.0.1".to_string(),
                is_dev_port: true,
            },
            PortInfo {
                port: 52345,
                protocol: Protocol::Tcp,
                state: PortState::Established,
                pid: None,
                process_name: None,
                command: None,
                local_address: "192.168.1.1".to_string(),
                is_dev_port: false,
            },
        ];

        let result = ScanResult::new(ports).filter_listening_only();
        assert_eq!(result.summary.total_ports, 1);
        assert_eq!(result.ports[0].port, 3000);
    }

    // TC-011: 開発ポートステータス
    #[test]
    fn test_dev_port_status() {
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
        
        // 3000は使用中
        assert_eq!(result.dev_port_status.get("3000"), Some(&DevPortStatus::InUse));
        // 8080は利用可能
        assert_eq!(result.dev_port_status.get("8080"), Some(&DevPortStatus::Available));
    }

    // TC-008: scan_specific_ports - 特定ポート
    #[test]
    fn test_filter_by_ports() {
        let ports = vec![
            PortInfo {
                port: 3000,
                protocol: Protocol::Tcp,
                state: PortState::Listening,
                pid: None,
                process_name: None,
                command: None,
                local_address: "127.0.0.1".to_string(),
                is_dev_port: true,
            },
            PortInfo {
                port: 8080,
                protocol: Protocol::Tcp,
                state: PortState::Listening,
                pid: None,
                process_name: None,
                command: None,
                local_address: "0.0.0.0".to_string(),
                is_dev_port: true,
            },
            PortInfo {
                port: 22,
                protocol: Protocol::Tcp,
                state: PortState::Listening,
                pid: None,
                process_name: None,
                command: None,
                local_address: "0.0.0.0".to_string(),
                is_dev_port: false,
            },
        ];

        let result = ScanResult::new(ports).filter_by_ports(&[3000, 8080]);
        assert_eq!(result.summary.total_ports, 2);
        assert!(result.ports.iter().any(|p| p.port == 3000));
        assert!(result.ports.iter().any(|p| p.port == 8080));
        assert!(!result.ports.iter().any(|p| p.port == 22));
    }

    // TC-009: scan_port_range - ポート範囲
    #[test]
    fn test_filter_by_range() {
        let ports = vec![
            PortInfo {
                port: 2999,
                protocol: Protocol::Tcp,
                state: PortState::Listening,
                pid: None,
                process_name: None,
                command: None,
                local_address: "127.0.0.1".to_string(),
                is_dev_port: false,
            },
            PortInfo {
                port: 3000,
                protocol: Protocol::Tcp,
                state: PortState::Listening,
                pid: None,
                process_name: None,
                command: None,
                local_address: "127.0.0.1".to_string(),
                is_dev_port: true,
            },
            PortInfo {
                port: 3500,
                protocol: Protocol::Tcp,
                state: PortState::Listening,
                pid: None,
                process_name: None,
                command: None,
                local_address: "127.0.0.1".to_string(),
                is_dev_port: false,
            },
            PortInfo {
                port: 4001,
                protocol: Protocol::Tcp,
                state: PortState::Listening,
                pid: None,
                process_name: None,
                command: None,
                local_address: "127.0.0.1".to_string(),
                is_dev_port: false,
            },
        ];

        let result = ScanResult::new(ports).filter_by_range(3000, 4000);
        assert_eq!(result.summary.total_ports, 2);
        assert!(result.ports.iter().all(|p| p.port >= 3000 && p.port <= 4000));
    }
}
