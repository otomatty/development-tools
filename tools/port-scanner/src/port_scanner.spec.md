# Port Scanner Specification

## Related Files

- Implementation: `tools/port-scanner/src/`
- Issue: `docs/01_issues/open/2025_11/20251129_port_scanner.md`

## Requirements

### 責務

- ローカルマシンの使用中ポートをスキャン
- ポートを使用しているプロセス情報を取得
- 結果を JSON/テキスト形式で出力

### 状態構造

```rust
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

pub struct ScanResult {
    pub summary: ScanSummary,
    pub ports: Vec<PortInfo>,
    pub dev_port_status: HashMap<u16, DevPortStatus>,
}

pub struct ScanSummary {
    pub total_ports: usize,
    pub tcp_ports: usize,
    pub udp_ports: usize,
    pub dev_ports_in_use: usize,
}
```

### アクション

1. `scan_ports()` - 全ポートをスキャン
2. `scan_specific_ports(ports: &[u16])` - 指定ポートをスキャン
3. `scan_port_range(start: u16, end: u16)` - ポート範囲をスキャン
4. `scan_dev_ports()` - 開発用ポートのみスキャン
5. `filter_listening_only(result: ScanResult)` - LISTENING のみフィルタ

## Test Cases

### TC-001: scan_ports - 空の結果

- Given: ポートが使用されていない環境（モック）
- When: `scan_ports()`を実行
- Then: 空の ports リストと summary.total_ports == 0

### TC-002: parse_lsof_line - 正常な LISTENING 行

- Given: `node 12345 user 23u IPv4 0x... 0t0 TCP 127.0.0.1:3000 (LISTEN)`
- When: `parse_lsof_line()`を実行
- Then: port=3000, protocol=TCP, state=LISTENING, pid=12345, process_name="node"

### TC-003: parse_lsof_line - ESTABLISHED 行

- Given: `chrome 67890 user 50u IPv4 0x... 0t0 TCP 192.168.1.1:52345->142.250.196.46:443 (ESTABLISHED)`
- When: `parse_lsof_line()`を実行
- Then: port=52345, protocol=TCP, state=ESTABLISHED

### TC-004: parse_ss_line - Linux ss 出力

- Given: `LISTEN 0 128 127.0.0.1:3000 0.0.0.0:* users:(("node",pid=12345,fd=23))`
- When: `parse_ss_line()`を実行
- Then: port=3000, protocol=TCP, state=LISTENING, pid=12345

### TC-005: is_dev_port - 開発ポート判定

- Given: port=3000
- When: `is_dev_port(3000)`を実行
- Then: true

### TC-006: is_dev_port - 非開発ポート判定

- Given: port=22
- When: `is_dev_port(22)`を実行
- Then: false

### TC-007: filter_by_protocol - TCP のみフィルタ

- Given: TCP/UDP ポートを含む ScanResult
- When: `filter_by_protocol(result, Protocol::Tcp)`を実行
- Then: TCP ポートのみ残る

### TC-008: scan_specific_ports - 特定ポート

- Given: ポート 3000, 8080 を指定
- When: `scan_specific_ports(&[3000, 8080])`を実行
- Then: 指定ポートの情報のみ返却

### TC-009: scan_port_range - ポート範囲

- Given: 範囲 3000-4000 を指定
- When: `scan_port_range(3000, 4000)`を実行
- Then: 範囲内のポート情報のみ返却

### TC-010: JSON 出力形式

- Given: ScanResult オブジェクト
- When: JSON シリアライズ
- Then: 期待する JSON 構造

### TC-011: 開発ポートステータス

- Given: 3000 が使用中、8080 が空き
- When: `get_dev_port_status()`を実行
- Then: 3000="in_use", 8080="available"
