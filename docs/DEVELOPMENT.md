# 開発ガイド

Development Tools の開発環境セットアップと日常的な開発タスクについて説明します。

---

## 📋 目次

- [前提条件](#前提条件)
- [環境構築](#環境構築)
- [開発サーバー](#開発サーバー)
- [ビルド](#ビルド)
- [テスト](#テスト)
- [デバッグ](#デバッグ)
- [よくある問題と解決方法](#よくある問題と解決方法)

---

## 前提条件

### 必須ツール

| ツール                                                   | バージョン | 用途                         |
| -------------------------------------------------------- | ---------- | ---------------------------- |
| [Rust](https://rustup.rs/)                               | 1.70+      | バックエンド、フロントエンド |
| [Node.js](https://nodejs.org/) or [Bun](https://bun.sh/) | 18+ / 1.0+ | パッケージ管理、Tailwind CSS |
| [Trunk](https://trunkrs.dev/)                            | 0.18+      | Rust WASM ビルド             |

### オプション

| ツール                                                          | 用途                     |
| --------------------------------------------------------------- | ------------------------ |
| [Tauri CLI](https://tauri.app/v1/guides/getting-started/setup/) | Tauri コマンドライン操作 |
| [SQLite Browser](https://sqlitebrowser.org/)                    | データベース確認         |

---

## 環境構築

### 1. Rust のインストール

```bash
# rustupでRustをインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# シェルを再起動するか、環境変数を読み込み
source $HOME/.cargo/env

# バージョン確認
rustc --version
cargo --version
```

### 2. WASM ターゲットの追加

```bash
rustup target add wasm32-unknown-unknown
```

### 3. Trunk のインストール

```bash
cargo install trunk
```

### 4. Node.js/Bun のインストール

```bash
# Bunの場合（推奨）
curl -fsSL https://bun.sh/install | bash

# または Node.js（nvmを使用）
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
```

### 5. プロジェクトの依存関係をインストール

```bash
# リポジトリのクローン
git clone https://github.com/otomatty/development-tools.git
cd development-tools

# npm依存関係（Tailwind CSS）
bun install
# または
npm install
```

### 6. 環境変数の設定

プロジェクトルートに`.env`ファイルを作成：

```bash
# .env
GITHUB_CLIENT_ID=your_github_oauth_app_client_id
```

#### GitHub OAuth App の作成

1. [GitHub Developer Settings](https://github.com/settings/developers)にアクセス
2. "New OAuth App"をクリック
3. 以下の情報を入力：
   - **Application name**: Development Tools
   - **Homepage URL**: `http://localhost:1420`
   - **Authorization callback URL**: `development-tools://callback`
4. Client ID をコピーして`.env`に設定

### 7. CLI ツールのビルド

```bash
# 全ツールをビルド
for tool in todo-collector shai-hulud-scanner loc-counter large-file-finder port-scanner; do
  echo "Building $tool..."
  cd tools/$tool && cargo build --release && cd ../..
done
```

---

## 開発サーバー

### Tauri アプリを起動（推奨）

```bash
bun run dev
# または
npm run dev
```

これにより以下が起動します：

- Trunk によるフロントエンドのホットリロード
- Tauri によるデスクトップウィンドウ
- Tailwind CSS のウォッチモード

### フロントエンドのみ起動

```bash
bun run dev:frontend
# または
trunk serve
```

ブラウザで`http://localhost:1420`にアクセス可能（ただし Tauri API は使用不可）

### CSS のウォッチモード

別ターミナルで CSS の変更を監視：

```bash
bun run watch:css
```

---

## ビルド

### 開発ビルド

```bash
bun run build:debug
```

### リリースビルド

```bash
bun run build
```

ビルド成果物は以下に出力されます：

- **macOS**: `target/release/bundle/dmg/`
- **Windows**: `target/release/bundle/msi/`
- **Linux**: `target/release/bundle/deb/`, `target/release/bundle/appimage/`

### フロントエンドのみビルド

```bash
trunk build --release
```

出力: `dist/`

---

## テスト

### バックエンドテスト

```bash
# すべてのテストを実行
cd src-tauri && cargo test

# 特定のテストを実行
cargo test test_add_xp

# 特定のモジュールのテストを実行
cargo test --package development-tools database::

# 出力を表示
cargo test -- --nocapture
```

### コードチェック

```bash
# フロントエンドのコンパイルチェック
cargo check --package development-tools-ui

# バックエンドのコンパイルチェック
cargo check --package development-tools

# Clippyによる静的解析
cargo clippy --all-targets
```

### フォーマット

```bash
# コードフォーマット
cargo fmt

# フォーマットチェック（CI用）
cargo fmt -- --check
```

---

## デバッグ

### ログの確認

#### バックエンド（Tauri/Rust）

`eprintln!`マクロでターミナルに出力：

```rust
eprintln!("Debug: user_id = {}", user_id);
```

#### フロントエンド（Leptos/WASM）

`web_sys::console`を使用：

```rust
web_sys::console::log_1(&format!("Debug: value = {:?}", value).into());
web_sys::console::error_1(&format!("Error: {}", e).into());
```

### データベースの確認

SQLite データベースは以下の場所に保存されます：

```
# macOS
~/Library/Application Support/com.development-tools/development_tools.db

# Linux
~/.local/share/com.development-tools/development_tools.db

# Windows
%APPDATA%\com.development-tools\development_tools.db
```

SQLite Browser や`sqlite3`コマンドで内容を確認できます：

```bash
sqlite3 ~/Library/Application\ Support/com.development-tools/development_tools.db
sqlite> .tables
sqlite> SELECT * FROM users;
sqlite> .quit
```

### DevTools の使用

開発モードでは、ブラウザの DevTools が利用可能です：

1. アプリを起動
2. `Cmd + Option + I`（macOS）または`Ctrl + Shift + I`（Windows/Linux）
3. Console、Network、Elements タブで確認

---

## よくある問題と解決方法

### 1. `trunk serve`でエラーが発生する

**症状**: `error: could not compile 'development-tools-ui'`

**解決方法**:

```bash
# WASMターゲットが追加されているか確認
rustup target list | grep wasm32

# 追加されていない場合
rustup target add wasm32-unknown-unknown

# キャッシュをクリアして再ビルド
cargo clean
trunk serve
```

### 2. GitHub 認証が動作しない

**症状**: ログインボタンをクリックしても何も起きない

**解決方法**:

1. `.env`ファイルが存在するか確認
2. `GITHUB_CLIENT_ID`が正しく設定されているか確認
3. ターミナルで以下を確認：
   ```
   GitHub Client ID loaded: XXXXXXXX...
   ```
   が表示されていない場合、環境変数が読み込まれていません

### 3. データベースエラー

**症状**: `Database error: ...`

**解決方法**:

```bash
# データベースファイルを削除してリセット
rm ~/Library/Application\ Support/com.development-tools/development_tools.db

# アプリを再起動
```

### 4. ツールが見つからない

**症状**: ツール一覧が空

**解決方法**:

```bash
# CLIツールがビルドされているか確認
ls tools/*/target/release/

# ビルドされていない場合は再ビルド
cd tools/todo-collector && cargo build --release
```

### 5. Tailwind CSS が反映されない

**症状**: スタイルが適用されない

**解決方法**:

```bash
# CSSを再ビルド
bun run build:css

# または監視モードで起動
bun run watch:css
```

---

## 開発フロー

### 新機能の追加

1. **Issue の作成**: `docs/01_issues/open/YYYY_MM/`に Issue ファイルを作成
2. **ブランチの作成**: `git checkout -b feature/issue-XXX-description`
3. **実装**: コードを実装
4. **テスト**: `cargo test`でテストを実行
5. **ドキュメント更新**: 必要に応じてドキュメントを更新
6. **PR 作成**: GitHub でプルリクエストを作成
7. **ログ記録**: `docs/05_logs/`に作業ログを記録
8. **Issue 解決**: `docs/01_issues/resolved/`に Issue を移動

### コードレビューのポイント

- [ ] コードは規約に従っているか
- [ ] テストが追加されているか
- [ ] ドキュメントが更新されているか
- [ ] パフォーマンスへの影響はないか
- [ ] セキュリティの考慮はあるか

---

## 参考リンク

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Leptos Documentation](https://leptos.dev/)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)
- [Rust Book](https://doc.rust-lang.org/book/)
