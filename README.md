# Development Tools

<div align="center">

**CLI 開発ツールを GUI で簡単に操作できるデスクトップアプリケーション**

[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri)](https://tauri.app/)
[![Leptos](https://img.shields.io/badge/Leptos-0.7-orange)](https://leptos.dev/)
[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

</div>

---

## 📖 概要

Development Tools は、Rust 製の CLI ツールを GUI から簡単に実行できるデスクトップアプリケーションです。

### 主な機能

- **🔧 CLI ツールの GUI ラッパー** - CLI オプションを覚える必要なく、フォームで視覚的に設定・実行
- **🎮 GitHub ゲーミフィケーション** - レベルシステム、バッジ、ストリーク、チャレンジで開発を楽しく
- **🌐 モックサーバー** - 静的ファイル配信サーバーをワンクリックで起動
- **⚙️ カスタマイズ可能な設定** - アニメーション、通知、同期間隔などを自由に設定

---

## 🛠️ 同梱ツール

| ツール                 | 説明                                                   | カテゴリ     |
| ---------------------- | ------------------------------------------------------ | ------------ |
| **TODO Collector**     | コード内の TODO/FIXME/HACK コメントを収集・一覧化      | Code Quality |
| **Shai-Hulud Scanner** | npm サプライチェーン攻撃の影響を受けたパッケージを検出 | Security     |
| **LOC Counter**        | 言語別コード行数をカウント・統計表示                   | Analytics    |
| **Large File Finder**  | 閾値以上の行数を持つファイルを検出                     | Analytics    |
| **Port Scanner**       | ローカルで使用中のポートとプロセスを確認               | Utility      |

---

## 🎮 ゲーミフィケーション機能

GitHub 連携により、開発活動を可視化・ゲーミフィケーション化：

- **📈 レベルシステム** - コミット、PR、Issue 作成で XP を獲得しレベルアップ
- **🏆 バッジシステム** - 特定の条件を達成するとバッジを獲得
- **🔥 ストリーク** - 連続コミット日数を追跡
- **🎯 チャレンジ** - デイリー/ウィークリーチャレンジで目標を設定
- **📊 コントリビューショングラフ** - GitHub 風の活動グラフを表示

---

## 📋 技術スタック

| レイヤー              | 技術                                          |
| --------------------- | --------------------------------------------- |
| **フレームワーク**    | [Tauri 2.0](https://tauri.app/)               |
| **フロントエンド**    | [Leptos 0.7](https://leptos.dev/) (Rust/WASM) |
| **スタイリング**      | [Tailwind CSS 3](https://tailwindcss.com/)    |
| **データベース**      | SQLite (sqlx)                                 |
| **HTTP クライアント** | reqwest                                       |
| **HTTP サーバー**     | axum + tower-http                             |

---

## 🚀 セットアップ

### 前提条件

以下のツールがインストールされている必要があります：

- [Rust](https://rustup.rs/) (1.70 以上)
- [Node.js](https://nodejs.org/) または [Bun](https://bun.sh/)
- [Trunk](https://trunkrs.dev/) - Rust WASM ビルドツール

```bash
# Rustのインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASMターゲットの追加
rustup target add wasm32-unknown-unknown

# Trunkのインストール
cargo install trunk

# Tauri CLIのインストール（オプション）
cargo install tauri-cli
```

### インストール

```bash
# リポジトリのクローン
git clone https://github.com/otomatty/development-tools.git
cd development-tools

# Git hooksのセットアップ（初回のみ）
./scripts/setup-hooks.sh

# 依存関係のインストール
bun install
# または
npm install

# CLIツールのビルド
cd tools/todo-collector && cargo build --release && cd ../..
cd tools/shai-hulud-scanner && cargo build --release && cd ../..
cd tools/loc-counter && cargo build --release && cd ../..
cd tools/large-file-finder && cargo build --release && cd ../..
cd tools/port-scanner && cargo build --release && cd ../..
```

### 環境変数の設定

GitHub 連携機能を使用する場合は、`.env`ファイルを作成します：

```bash
# .env
GITHUB_CLIENT_ID=your_github_oauth_app_client_id
```

> **Note**: GitHub OAuth App を[Developer Settings](https://github.com/settings/developers)から作成し、Client ID を取得してください。

### 開発サーバーの起動

```bash
# 開発モードで起動（ホットリロード対応）
bun run dev
# または
npm run dev
```

### 本番ビルド

```bash
# リリースビルド
bun run build
# または
npm run build
```

---

## 📁 ディレクトリ構造

```
development-tools/
├── src/                      # フロントエンド（Leptos/WASM）
│   ├── app.rs                # メインアプリケーション
│   ├── components/           # UIコンポーネント
│   │   ├── home/             # ホーム画面（ゲーミフィケーション）
│   │   ├── settings/         # 設定ページ
│   │   └── mock_server/      # モックサーバーUI
│   ├── types/                # 型定義
│   └── tauri_api.rs          # Tauri IPC呼び出し
│
├── src-tauri/                # バックエンド（Tauri/Rust）
│   └── src/
│       ├── commands/         # Tauriコマンド
│       ├── database/         # SQLite操作
│       ├── github/           # GitHub API クライアント
│       ├── auth/             # OAuth認証
│       └── mock_server/      # HTTPサーバー
│
├── tools/                    # CLIツール群
│   ├── todo-collector/
│   ├── shai-hulud-scanner/
│   ├── loc-counter/
│   ├── large-file-finder/
│   └── port-scanner/
│
├── docs/                     # ドキュメント
│   ├── requirements.md       # 要件定義
│   ├── prd/                  # PRD（製品要件定義）
│   ├── 01_issues/            # Issue管理
│   ├── 03_plans/             # 実装計画
│   └── 05_logs/              # 作業ログ
│
└── public/                   # 静的ファイル
```

---

## 📜 利用可能なコマンド

| コマンド               | 説明                                     |
| ---------------------- | ---------------------------------------- |
| `bun run dev`          | 開発サーバーを起動（ホットリロード対応） |
| `bun run build`        | リリースビルドを作成                     |
| `bun run build:debug`  | デバッグビルドを作成                     |
| `bun run build:css`    | Tailwind CSS をビルド                    |
| `bun run watch:css`    | Tailwind CSS をウォッチモードでビルド    |
| `bun run dev:frontend` | フロントエンドのみ起動（Trunk）          |

---

## 🧪 テスト

```bash
# バックエンドのテスト
cd src-tauri && cargo test

# 特定のテストを実行
cargo test --package development-tools -- test_name
```

---

## 📖 ドキュメント

詳細なドキュメントは`docs/`ディレクトリを参照してください：

- [要件定義](docs/requirements.md)
- [開発ガイド](docs/DEVELOPMENT.md)
- [アーキテクチャ](docs/ARCHITECTURE.md)
- [コントリビューションガイド](CONTRIBUTING.md)

### PRD（製品要件定義）

- [ホーム画面（ゲーミフィケーション）](docs/prd/home-gamification.md)
- [モックサーバー](docs/prd/mock-server.md)
- [設定ページ](docs/prd/settings-page.md)

---

## 🤝 コントリビューション

コントリビューションを歓迎します！詳細は[CONTRIBUTING.md](CONTRIBUTING.md)を参照してください。

1. このリポジトリをフォーク
2. フィーチャーブランチを作成 (`git checkout -b feature/issue-XXX-amazing-feature`)
3. 変更をコミット (`git commit -m 'feat: Add amazing feature'`)
4. ブランチをプッシュ (`git push origin feature/issue-XXX-amazing-feature`)
5. プルリクエストを作成

---

## 📄 ライセンス

このプロジェクトは MIT ライセンスの下で公開されています。詳細は[LICENSE](LICENSE)を参照してください。

---

## 👤 作者

- **sugaiakimasa** - [@otomatty](https://github.com/otomatty)

---

## 💡 推奨 IDE 設定

[VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
