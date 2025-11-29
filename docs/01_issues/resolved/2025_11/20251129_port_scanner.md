# Port Scanner ツール実装

## Issue ID

GitHub Issue #3

## 概要

ローカルマシンで使用中のネットワークポートを検出し、どのプロセスがどのポートを使用しているかを表示する CLI ツールを実装する。

## 背景

開発中に「ポートが既に使用されています」エラーに遭遇することは多い。このツールにより、使用中のポートとプロセスを素早く確認し、問題を特定できる。

## 対応 OS

- macOS（優先）
- Linux（優先）
- Windows（将来対応）

## 機能要件

### 基本機能

- [ ] 使用中の TCP/UDP ポートを一覧表示
- [ ] ポートを使用しているプロセス名・PID を表示
- [ ] 特定ポートの使用状況確認
- [ ] 開発でよく使われるポート（3000, 5173, 8080 等）の状態確認

### CLI オプション

| オプション       | 説明                                             |
| ---------------- | ------------------------------------------------ |
| `--port`, `-p`   | 特定ポートをチェック（カンマ区切りで複数指定可） |
| `--range`, `-r`  | ポート範囲（例: 3000-4000）                      |
| `--protocol`     | プロトコル（tcp/udp/both）                       |
| `--output`, `-o` | 出力形式（text/json）                            |
| `--dev-ports`    | 開発用ポートのみ表示                             |
| `--listening`    | LISTENING ポートのみ表示                         |

## 技術仕様

### macOS

- `lsof -i -P -n` コマンドを使用
- 出力をパースしてポート情報を取得

### Linux

- `ss -tulnp` または `netstat -tulnp` を使用
- `/proc/net/tcp`, `/proc/net/udp` も検討

## 関連ファイル

- Spec: `tools/port-scanner/src/port_scanner.spec.md`
- 実装: `tools/port-scanner/`

## 完了条件

- [ ] cargo build --release が成功する
- [ ] cargo test が全てパスする
- [ ] macOS で動作確認
- [ ] Linux で動作確認
- [ ] JSON 出力が正しいフォーマット
- [ ] GUI から呼び出し可能
