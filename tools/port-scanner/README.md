# Port Scanner

ローカルマシンで使用中のネットワークポートを検出し、どのプロセスがどのポートを使用しているかを表示する CLI ツール。

## 特徴

- 🔍 使用中の TCP/UDP ポートを一覧表示
- 🏷️ ポートを使用しているプロセス名・PID を表示
- 🎯 特定ポートの使用状況確認
- 🛠️ 開発でよく使われるポート（3000, 5173, 8080 等）の状態を素早く確認
- 📊 JSON/テキスト形式での出力

## 対応 OS

- ✅ macOS（`lsof`コマンド使用）
- ✅ Linux（`ss`コマンド使用）

## インストール

```bash
cd tools/port-scanner
cargo build --release
```

## 使い方

### 基本的な使用法

```bash
# 全ポートをスキャン（テキスト出力）
./target/release/port-scanner

# LISTENINGポートのみ表示
./target/release/port-scanner --listening

# 開発用ポートのみ表示
./target/release/port-scanner --dev-ports

# JSON形式で出力
./target/release/port-scanner --output json
```

### オプション

| オプション    | 短縮形 | 説明                                 | デフォルト |
| ------------- | ------ | ------------------------------------ | ---------- |
| `--port`      | `-p`   | 特定ポートをチェック（カンマ区切り） | -          |
| `--range`     | `-r`   | ポート範囲（例: 3000-4000）          | -          |
| `--protocol`  | -      | プロトコル（tcp/udp/both）           | both       |
| `--output`    | `-o`   | 出力形式（text/json）                | text       |
| `--dev-ports` | -      | 開発用ポートのみ表示                 | false      |
| `--listening` | -      | LISTENING ポートのみ表示             | false      |

### 使用例

```bash
# 特定のポートをチェック
./target/release/port-scanner --port 3000,8080,5173

# ポート範囲でスキャン
./target/release/port-scanner --range 3000-4000

# TCPポートのみ、LISTENING状態のみ
./target/release/port-scanner --protocol tcp --listening

# 開発ポートをJSON形式で出力
./target/release/port-scanner --dev-ports --output json
```

## 出力例

### テキスト出力

```
════════════════════════════════════════════════════════════════════════════════
 🔍 Port Scanner Results
════════════════════════════════════════════════════════════════════════════════

📊 Summary
  Total Ports:      17
  TCP Ports:        17
  UDP Ports:        0
  Dev Ports In Use: 2

🛠️  Dev Port Status
   3000 🟢 AVAILABLE
   5000 🔴 IN USE (ControlCe)
   5173 🟢 AVAILABLE
   8080 🟢 AVAILABLE

📋 Port Details
  PORT   PROTO  STATE        PROCESS         PID      ADDRESS
  ----------------------------------------------------------------------
  5000   TCP    LISTENING    ControlCe       710      *
  27017  TCP    LISTENING    mongod          983      127.0.0.1
```

### JSON 出力

```json
{
  "summary": {
    "total_ports": 2,
    "tcp_ports": 2,
    "udp_ports": 0,
    "dev_ports_in_use": 2
  },
  "ports": [
    {
      "port": 5000,
      "protocol": "tcp",
      "state": "LISTENING",
      "pid": 710,
      "process_name": "ControlCe",
      "command": null,
      "local_address": "*",
      "is_dev_port": true
    }
  ],
  "dev_port_status": {
    "5000": "in_use",
    "3000": "available",
    "8080": "available"
  }
}
```

## 開発用ポート一覧

デフォルトで監視される開発用ポート：

| ポート | 一般的な用途          |
| ------ | --------------------- |
| 3000   | React, Express, Rails |
| 3001   | Next.js (alt)         |
| 4200   | Angular               |
| 4321   | Astro                 |
| 5000   | Flask, ASP.NET        |
| 5173   | Vite                  |
| 5432   | PostgreSQL            |
| 6379   | Redis                 |
| 8000   | Django, PHP           |
| 8080   | Tomcat, 汎用          |
| 8888   | Jupyter               |
| 1420   | Tauri                 |
| 3306   | MySQL                 |
| 27017  | MongoDB               |

## ライセンス

MIT
