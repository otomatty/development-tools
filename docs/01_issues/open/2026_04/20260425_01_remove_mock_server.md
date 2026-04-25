# モックサーバー機能の完全削除

## 概要

本アプリの中核UXを「個人開発者のアクティビティ・コーチ」（GitHub 連携によるゲーミフィケーション + 活動可視化 + Issue / プロジェクト管理）に絞り込むため、性格の異なるユーティリティであるモックサーバー機能を完全に削除する。

## 背景・目的

### 削除する理由

- アプリの中核UX（ゲーミフィケーション + GitHub 活動把握 + プロジェクト管理）と **データソースが異なる**（モックサーバーは GitHub データに依存しない静的ファイル配信）
- Mockoon、json-server、Vite dev server 等の **競合が多く差別化が難しい** 領域
- 「毎日開いて作業ループに入る」体験との接続が弱く、UX の物語を分散させている
- バックエンド実装は完了しているが UI は未着手 — 削除コストが比較的低い今のうちに整理する

### 残す価値

なし（将来別プラグインとして切り出す可能性は残すが、本リポジトリからは完全に除去する）。

## 影響範囲

### Backend (Rust)

- [ ] `src-tauri/src/mock_server/` ディレクトリ全削除
  - `mod.rs`, `server.rs`, `repository.rs`, `types.rs`
- [ ] `src-tauri/src/commands/mock_server.rs` 削除
- [ ] `src-tauri/src/commands/mod.rs` から `mock_server` モジュール参照削除
- [ ] `src-tauri/src/lib.rs` (または `main.rs`) から Tauri command 登録削除
  - `start_mock_server`, `stop_mock_server`, `get_mock_server_state`,
    `create_mapping`, `update_mapping`, `delete_mapping`, `update_config`,
    `get_access_logs`, `clear_access_logs`, `list_files` 等
- [ ] `Cargo.toml` から不要になった依存を削除（要調査）
  - `axum`, `tower-http` がモックサーバー専用なら削除
  - 他の機能で使っている場合は残す

### Database

- [ ] マイグレーションファイルから以下のテーブル定義を削除
  - `mock_server_config`
  - `mock_server_mappings`
  - `mock_server_logs`
- [ ] 既存ユーザーの DB に対する drop マイグレーションを追加（新規マイグレーションとして）

### Frontend (TypeScript / React)

- [ ] `src/pages/MockServer/` ディレクトリ全削除
- [ ] `src/components/features/mock_server/` ディレクトリ全削除
- [ ] `src/components/pages/mock_server/` ディレクトリ全削除
- [ ] `src/components/mock_server/` ディレクトリ全削除（重複の可能性あり）
- [ ] ルーティング設定からモックサーバーページのルート削除
- [ ] サイドバー/ナビゲーションからメニュー項目削除
- [ ] Tauri invoke 呼び出しのラッパー関数（`src/lib/tauri.ts` 等）から削除

### Documentation

- [ ] `docs/prd/mock-server.md` 削除
- [ ] `docs/api/TAURI_COMMANDS.md` からモックサーバー系コマンドの記述削除
- [ ] `docs/database/SCHEMA.md` からモックサーバー系テーブル定義削除
- [ ] `docs/ARCHITECTURE.md` 内のモックサーバー言及箇所修正
- [ ] `docs/requirements.md` の機能一覧から削除
- [ ] `README.md` の機能一覧から削除

### Tests

- [ ] モックサーバー関連の単体テスト・統合テスト全削除

## 受け入れ条件

- [ ] `cargo build` がエラーなく通る
- [ ] `cargo test` がエラーなく通る（テスト数が想定通り減っている）
- [ ] フロントエンドのビルド (`pnpm build` / `npm run build`) が通る
- [ ] アプリを起動してホーム画面・設定画面に遷移しても警告・エラーが出ない
- [ ] 既存のユーザーDBがある状態でも起動時にマイグレーションが正常に走る
- [ ] `grep -r "mock_server\|MockServer\|mockServer" src src-tauri docs` で参照が残っていない
  （変数名・コメント含めて完全削除）

## 非スコープ

- モックサーバーの別リポジトリへの切り出し（やるなら別タスク）
- 削除に伴う他機能のリファクタ（最小限の参照削除のみ）

## 関連

- 後続: GitHub 連携の現状調査
- 後続: 具体的な UI の再設計
