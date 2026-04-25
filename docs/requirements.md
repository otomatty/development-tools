# Development Tools - 要件定義書

> ⚠️ **注意**: 本書は当初構想時の要件定義書です。CLIツールをGUIから実行する機能は現在のコードベースから削除されており、現行プロダクトはGitHub連携によるゲーミフィケーションを中心としています。`tools/` ディレクトリ配下のCLIツールはコマンドラインから単体で実行する独立したRust製ツールとして残っています。
>
> 最終更新日: 2025年11月26日

## 1. プロダクト概要

### 1.1 プロダクト名

**Development Tools**

### 1.2 目的

`tools/` ディレクトリ配下にあるRust製CLIツールをGUIから簡単に実行できるデスクトップアプリケーション。CLIのオプションを覚える必要がなく、実行結果を視覚的にわかりやすく表示する。

### 1.3 ターゲットユーザー

- チーム・組織内の開発者
- CLIツールのオプションを毎回調べるのが面倒な人
- 実行結果を視覚的に把握したい人

### 1.4 解決する課題

| 課題 | 解決策 |
|------|--------|
| CLIオプションを覚えるのが辛い | GUIフォームで視覚的にオプションを設定 |
| 実行結果がわかりにくい | サマリービュー・詳細ビューで視覚的に表示 |
| チームでの設定共有が難しい | GitHubリポジトリでプリセット・設定を共有（Phase 2） |

---

## 2. 技術スタック

| 項目 | 技術 |
|------|------|
| フレームワーク | Tauri 2.0 |
| フロントエンド | Leptos (Rust) |
| スタイリング | Tailwind CSS |
| 設定ファイル形式 | JSON |
| 対象プラットフォーム | デスクトップ（macOS優先） |

---

## 3. 機能要件

### 3.1 Phase 1（MVP）

| ID | 機能 | 説明 | 優先度 |
|----|------|------|--------|
| F1 | ツール一覧表示 | `tools/` 配下のツールをカード形式で一覧表示 | 必須 |
| F2 | ツール詳細・フォーム入力 | 選択したツールのオプションをフォームで設定・実行 | 必須 |
| F3 | リアルタイムログ表示 | ツール実行中の標準出力/エラー出力をリアルタイム表示 | 必須 |
| F4 | 結果表示（サマリー） | 実行結果の要約をカード・数値で視覚的に表示 | 必須 |
| F5 | 結果表示（詳細） | 個別の検出結果をテーブル/リスト形式で表示 | 必須 |

### 3.2 Phase 2（将来実装）

| ID | 機能 | 説明 | 優先度 |
|----|------|------|--------|
| F6 | プリセット機能 | よく使うオプション組み合わせを保存・呼び出し | 高 |
| F7 | 実行履歴 | 過去1ヶ月の実行履歴を保存・参照 | 中 |
| F8 | GitHub同期 | プリセット・設定をGitHubリポジトリで共有 | 中 |

---

## 4. ツール設定ファイル仕様

### 4.1 配置場所

各ツールのディレクトリ内に `tool.json` を配置する。

```
tools/
└── shai-hulud-scanner/
    ├── Cargo.toml
    ├── tool.json          ← ツール設定ファイル
    └── src/
        └── ...
```

### 4.2 設定ファイルスキーマ

```json
{
  "$schema": "https://example.com/tool-schema.json",
  "name": "shai-hulud-scanner",
  "displayName": "Shai-Hulud Scanner",
  "description": "npmサプライチェーン攻撃の影響を受けたパッケージを検出",
  "version": "1.0.0",
  "binary": "target/release/shai-hulud-scanner",
  "icon": "shield",
  "category": "security",
  "options": [
    {
      "name": "scan-dir",
      "flag": "--scan-dir",
      "shortFlag": "-s",
      "type": "path",
      "description": "スキャン対象ディレクトリ",
      "placeholder": "~/projects",
      "required": false
    },
    {
      "name": "current-dir",
      "flag": "--current-dir",
      "type": "boolean",
      "description": "カレントディレクトリのみスキャン",
      "default": false
    },
    {
      "name": "output",
      "flag": "--output",
      "shortFlag": "-o",
      "type": "select",
      "description": "出力形式",
      "options": ["text", "json"],
      "default": "text"
    },
    {
      "name": "skip-global",
      "flag": "--skip-global",
      "type": "boolean",
      "description": "グローバルパッケージのスキャンをスキップ",
      "default": false
    },
    {
      "name": "offline",
      "flag": "--offline",
      "type": "boolean",
      "description": "オフラインモード（キャッシュ使用）",
      "default": false
    }
  ],
  "resultParser": {
    "type": "json",
    "outputFlag": "--output json",
    "schema": {
      "summary": {
        "critical": "$.detections[?(@.severity=='CRITICAL')].length",
        "warning": "$.detections[?(@.severity=='WARNING')].length",
        "suspicious": "$.suspicious_files.length"
      },
      "details": {
        "items": "$.detections",
        "columns": ["package", "installed_version", "severity", "location"]
      }
    }
  }
}
```

### 4.3 スキーマ定義

#### ルートオブジェクト

| フィールド | 型 | 必須 | 説明 |
|------------|-----|------|------|
| `name` | string | ✅ | ツールの識別名（英数字・ハイフン） |
| `displayName` | string | ✅ | 表示名 |
| `description` | string | ✅ | ツールの説明 |
| `version` | string | ✅ | バージョン（semver形式） |
| `binary` | string | ✅ | 実行ファイルへの相対パス |
| `icon` | string | ❌ | アイコン名（後述のアイコン一覧参照） |
| `category` | string | ❌ | カテゴリ（security, utility, etc.） |
| `options` | Option[] | ✅ | コマンドラインオプションの配列 |
| `resultParser` | ResultParser | ❌ | 結果パーサーの設定 |

#### Option オブジェクト

| フィールド | 型 | 必須 | 説明 |
|------------|-----|------|------|
| `name` | string | ✅ | オプションの識別名 |
| `flag` | string | ✅ | CLIフラグ（例: `--scan-dir`） |
| `shortFlag` | string | ❌ | 短縮フラグ（例: `-s`） |
| `type` | OptionType | ✅ | オプションの型 |
| `description` | string | ✅ | オプションの説明 |
| `required` | boolean | ❌ | 必須かどうか（デフォルト: false） |
| `default` | any | ❌ | デフォルト値 |
| `placeholder` | string | ❌ | プレースホルダーテキスト |
| `options` | string[] | ❌ | type="select"の場合の選択肢 |

#### OptionType 一覧

| type | 説明 | UIコンポーネント |
|------|------|------------------|
| `string` | 文字列入力 | テキストフィールド |
| `path` | ファイル/ディレクトリパス | パス入力 + ファイルピッカー |
| `boolean` | フラグ | トグルスイッチ |
| `select` | 選択肢から選択 | ドロップダウン |
| `number` | 数値入力 | 数値フィールド |

#### ResultParser オブジェクト

| フィールド | 型 | 必須 | 説明 |
|------------|-----|------|------|
| `type` | "json" \| "text" | ✅ | 出力形式 |
| `outputFlag` | string | ❌ | JSON出力を有効にするフラグ |
| `schema` | object | ❌ | パース設定 |

### 4.4 アイコン一覧

| アイコン名 | 用途 |
|------------|------|
| `shield` | セキュリティツール |
| `search` | 検索・スキャンツール |
| `code` | コード関連ツール |
| `package` | パッケージ管理ツール |
| `terminal` | CLI/シェルツール |
| `settings` | 設定・構成ツール |
| `chart` | 分析・統計ツール |
| `file` | ファイル操作ツール |

---

## 5. UI設計

### 5.1 デザイン方針

- **ダッシュボード風**のモダンUI
- ダークテーマをデフォルトとする
- 視覚的にわかりやすいカード・アイコン使用
- Tailwind CSSによるレスポンシブ対応

### 5.2 カラーパレット

| 用途 | カラー |
|------|--------|
| 背景（メイン） | `#0f172a` (slate-900) |
| 背景（カード） | `#1e293b` (slate-800) |
| テキスト（メイン） | `#f8fafc` (slate-50) |
| テキスト（サブ） | `#94a3b8` (slate-400) |
| アクセント | `#3b82f6` (blue-500) |
| 成功 | `#22c55e` (green-500) |
| 警告 | `#eab308` (yellow-500) |
| エラー/Critical | `#ef4444` (red-500) |

### 5.3 画面構成

```
┌─────────────────────────────────────────────────────────────────────┐
│  🔧 Development Tools                                    [─][□][×] │
├───────────────┬─────────────────────────────────────────────────────┤
│               │                                                     │
│  TOOLS        │   Tool Detail / Execution                           │
│  ───────────  │   ───────────────────────────────────────────────   │
│               │                                                     │
│  🛡 Shai-     │   🛡 Shai-Hulud Scanner                             │
│    Hulud      │   ──────────────────────                            │
│    Scanner    │   npmサプライチェーン攻撃の影響を受けた              │
│               │   パッケージを検出                                   │
│  📦 Tool2     │                                                     │
│               │   ┌─ Options ─────────────────────────────────┐     │
│  🔍 Tool3     │   │                                           │     │
│               │   │ Scan Directory                            │     │
│               │   │ [~/projects                        ] [📁] │     │
│               │   │                                           │     │
│               │   │ ☐ Current directory only                  │     │
│               │   │ ☐ Skip global packages                    │     │
│               │   │ ☐ Offline mode                            │     │
│               │   │                                           │     │
│               │   │ Output Format                             │     │
│               │   │ [text                              ▼]     │     │
│               │   │                                           │     │
│               │   └───────────────────────────────────────────┘     │
│               │                                                     │
│               │   [ ▶ Run Scanner ]                                 │
│               │                                                     │
├───────────────┼─────────────────────────────────────────────────────┤
│               │   Output                                            │
│               │   ───────────────────────────────────────────────   │
│               │   [Summary] [Details] [Logs]                        │
│               │                                                     │
│               │   ┌───────────┐ ┌───────────┐ ┌───────────┐        │
│               │   │ CRITICAL  │ │  WARNING  │ │SUSPICIOUS │        │
│               │   │     1     │ │     3     │ │     0     │        │
│               │   │  ─────    │ │  ─────    │ │  ─────    │        │
│               │   │ 🔴 危険   │ │ 🟡 注意   │ │ 🟠 疑わしい│        │
│               │   └───────────┘ └───────────┘ └───────────┘        │
│               │                                                     │
└───────────────┴─────────────────────────────────────────────────────┘
```

### 5.4 画面一覧

| 画面ID | 画面名 | 説明 |
|--------|--------|------|
| S1 | サイドバー | 登録されたツールをアイコン付きリストで表示 |
| S2 | ツール詳細 | ツール情報とオプションフォーム |
| S3 | 実行ボタン | ツール実行トリガー |
| S4 | 結果 - サマリー | 重要指標をカードで表示 |
| S5 | 結果 - 詳細 | テーブル形式で個別結果表示 |
| S6 | 結果 - ログ | リアルタイムログ（ターミナル風） |

### 5.5 状態遷移

```
[ツール選択] → [オプション入力] → [実行] → [ログ表示] → [結果表示]
                     ↑                              │
                     └──────────── 再実行 ──────────┘
```

---

## 6. データフロー

### 6.1 ツール読み込み

```
1. アプリ起動
2. tools/ ディレクトリをスキャン
3. 各ツールの tool.json を読み込み
4. ツール一覧を構築
5. サイドバーに表示
```

### 6.2 ツール実行

```
1. ユーザーがオプションを入力
2. 「Run」ボタンをクリック
3. Tauri Commandを呼び出し
4. Rustバックエンドでプロセス起動
5. stdout/stderrをイベントでフロントエンドに送信
6. リアルタイムでログ表示
7. 実行完了後、結果をパース
8. サマリー・詳細ビューを更新
```

### 6.3 シーケンス図

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ Frontend │     │  Tauri   │     │  Rust    │     │   CLI    │
│ (Leptos) │     │  Core    │     │ Backend  │     │   Tool   │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │
     │ run_tool()     │                │                │
     │───────────────>│                │                │
     │                │ invoke()       │                │
     │                │───────────────>│                │
     │                │                │ spawn()        │
     │                │                │───────────────>│
     │                │                │                │
     │                │                │<─── stdout ────│
     │                │<── event ──────│                │
     │<── update ─────│                │                │
     │                │                │                │
     │                │                │<─── exit ──────│
     │                │<── result ─────│                │
     │<── result ─────│                │                │
     │                │                │                │
```

---

## 7. 非機能要件

### 7.1 パフォーマンス

| 項目 | 要件 |
|------|------|
| アプリ起動 | 3秒以内 |
| ツール起動 | 1秒以内 |
| ログ更新 | 100ms以内にUI反映 |
| メモリ使用量 | 200MB以下（アイドル時） |

### 7.2 エラーハンドリング

- CLIツールのエラーメッセージは**そのまま表示**する
- アプリ内部エラーはユーザーフレンドリーなメッセージを表示

### 7.3 データ保存

| データ | 保存場所 | 形式 |
|--------|----------|------|
| ツール設定 | `tools/*/tool.json` | JSON |
| アプリ設定 | OS標準設定ディレクトリ | JSON |
| 実行履歴（Phase 2） | OS標準データディレクトリ | SQLite |

---

## 8. 開発フェーズ

### 8.1 Phase 1（MVP）

**推定工数**: 2-3週間

| Week | タスク |
|------|--------|
| 1 | プロジェクト構造整備、Tailwind CSS設定、基本レイアウト |
| 1 | ツール設定ファイル（tool.json）パーサー実装 |
| 2 | サイドバー（ツール一覧）実装 |
| 2 | オプションフォームの動的生成 |
| 2 | ツール実行機能（Tauri Command） |
| 3 | リアルタイムログ表示 |
| 3 | 結果パーサー・サマリービュー |
| 3 | 詳細ビュー実装、テスト・バグ修正 |

### 8.2 Phase 2

**推定工数**: 2週間

| Week | タスク |
|------|--------|
| 1 | プリセット機能（保存・読み込み） |
| 1 | 実行履歴（SQLite、1ヶ月保存） |
| 2 | GitHub同期機能 |
| 2 | テスト・ドキュメント整備 |

---

## 9. ディレクトリ構造

```
development-tools/
├── .docs/
│   └── requirements.md          # 本ファイル
├── src/
│   ├── app.rs                   # メインアプリコンポーネント
│   ├── main.rs                  # エントリーポイント
│   └── components/              # UIコンポーネント（予定）
│       ├── sidebar.rs
│       ├── tool_detail.rs
│       ├── option_form.rs
│       ├── log_viewer.rs
│       └── result_view.rs
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   └── commands/            # Tauri Commands（予定）
│   │       ├── mod.rs
│   │       ├── tool_loader.rs
│   │       └── tool_runner.rs
│   └── tauri.conf.json
├── tools/
│   └── shai-hulud-scanner/
│       ├── tool.json            # ツール設定ファイル（作成予定）
│       ├── Cargo.toml
│       └── src/
├── styles.css
├── Cargo.toml
└── Trunk.toml
```

---

## 10. 付録

### 10.1 shai-hulud-scanner 用 tool.json サンプル

```json
{
  "name": "shai-hulud-scanner",
  "displayName": "Shai-Hulud Scanner",
  "description": "npmサプライチェーン攻撃（Shai-Hulud）の影響を受けたパッケージを検出するセキュリティスキャナー",
  "version": "1.0.0",
  "binary": "target/release/shai-hulud-scanner",
  "icon": "shield",
  "category": "security",
  "options": [
    {
      "name": "scan-dir",
      "flag": "--scan-dir",
      "shortFlag": "-s",
      "type": "path",
      "description": "スキャン対象ディレクトリ（デフォルト: ホームディレクトリ）",
      "placeholder": "~/projects",
      "required": false
    },
    {
      "name": "current-dir",
      "flag": "--current-dir",
      "type": "boolean",
      "description": "カレントディレクトリのみスキャン",
      "default": false
    },
    {
      "name": "csv-file",
      "flag": "--csv-file",
      "shortFlag": "-c",
      "type": "path",
      "description": "ローカルのCSVファイルパス",
      "required": false
    },
    {
      "name": "output",
      "flag": "--output",
      "shortFlag": "-o",
      "type": "select",
      "description": "出力形式",
      "options": ["text", "json"],
      "default": "json"
    },
    {
      "name": "verbose",
      "flag": "--verbose",
      "type": "boolean",
      "description": "スキャンした全パッケージを表示",
      "default": false
    },
    {
      "name": "offline",
      "flag": "--offline",
      "type": "boolean",
      "description": "オフラインモード（キャッシュされたCSVを使用）",
      "default": false
    },
    {
      "name": "skip-global",
      "flag": "--skip-global",
      "type": "boolean",
      "description": "グローバルパッケージのスキャンをスキップ",
      "default": false
    },
    {
      "name": "skip-suspicious",
      "flag": "--skip-suspicious",
      "type": "boolean",
      "description": "不審なファイルの検出をスキップ",
      "default": false
    }
  ],
  "resultParser": {
    "type": "json",
    "outputFlag": "--output json",
    "schema": {
      "summary": [
        {
          "key": "critical",
          "label": "Critical",
          "path": "$.detections[?(@.severity=='CRITICAL')]",
          "countType": "length",
          "color": "red",
          "icon": "alert-circle"
        },
        {
          "key": "warning",
          "label": "Warning",
          "path": "$.detections[?(@.severity=='WARNING')]",
          "countType": "length",
          "color": "yellow",
          "icon": "alert-triangle"
        },
        {
          "key": "suspicious",
          "label": "Suspicious",
          "path": "$.suspicious_files",
          "countType": "length",
          "color": "orange",
          "icon": "file-warning"
        }
      ],
      "details": {
        "items": "$.detections",
        "columns": [
          { "key": "package", "label": "Package", "width": "200px" },
          { "key": "installed_version", "label": "Version", "width": "100px" },
          { "key": "severity", "label": "Severity", "width": "100px" },
          { "key": "source", "label": "Source", "width": "100px" },
          { "key": "location", "label": "Location", "flex": 1 }
        ]
      }
    }
  }
}
```

---

## 11. 変更履歴

| 日付 | バージョン | 変更内容 |
|------|------------|----------|
| 2025-11-26 | 1.0.0 | 初版作成 |

