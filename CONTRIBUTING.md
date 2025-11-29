# コントリビューションガイド

Development Tools へのコントリビューションを歓迎します！このドキュメントでは、プロジェクトへの貢献方法について説明します。

---

## 📋 目次

- [行動規範](#行動規範)
- [開発の始め方](#開発の始め方)
- [Issue 管理](#issue管理)
- [ブランチ戦略](#ブランチ戦略)
- [コミットメッセージ規約](#コミットメッセージ規約)
- [プルリクエスト](#プルリクエスト)
- [コーディング規約](#コーディング規約)
- [ドキュメント更新](#ドキュメント更新)

---

## 行動規範

すべてのコントリビューターは、互いに敬意を持って接することが期待されます。ハラスメント、差別、攻撃的な言動は許容されません。

---

## 開発の始め方

### 1. リポジトリのフォーク

GitHub でこのリポジトリをフォークしてください。

### 2. ローカルにクローン

```bash
git clone https://github.com/YOUR_USERNAME/development-tools.git
cd development-tools
```

### 3. アップストリームの設定

```bash
git remote add upstream https://github.com/otomatty/development-tools.git
```

### 4. 環境構築

[README.md](README.md)のセットアップ手順に従って開発環境を構築してください。

---

## Issue 管理

### Issue の作成場所

- **GitHub Issues**: バグ報告、機能要望、一般的な質問
- **docs/01_issues/open/**: 詳細な技術的 Issue（実装計画を含む）

### ローカル Issue の構造

```
docs/01_issues/
├── open/           # 未解決のIssue
│   └── 2025_11/    # 月別ディレクトリ
│       └── YYYYMMDD_XX_issue-name.md
└── resolved/       # 解決済みのIssue
    └── 2025_11/
```

### Issue ファイルのテンプレート

```markdown
# Issue: [タイトル]

## 概要

[問題や要望の簡潔な説明]

## 現状

[現在の状態]

## 要件

### 機能要件

1. [要件 1]
2. [要件 2]

### 技術要件

1. [技術要件 1]
2. [技術要件 2]

## 影響範囲

### 新規ファイル

- [ファイルパス]

### 修正ファイル

- [ファイルパス] - [変更内容]

## テストケース

1. **TC-001**: [テストケースの説明]
```

---

## ブランチ戦略

### ブランチ命名規則

```
feature/issue-XXX-short-description
bugfix/issue-XXX-short-description
docs/short-description
refactor/short-description
```

### 例

```bash
git checkout -b feature/issue-74-code-stats-phase1
git checkout -b bugfix/issue-42-login-error
git checkout -b docs/update-readme
git checkout -b refactor/cleanup-database-module
```

### メインブランチ

- `main`: 本番リリース用ブランチ
- `develop`: 開発用ブランチ（存在する場合）

---

## コミットメッセージ規約

[Conventional Commits](https://www.conventionalcommits.org/)に従います。

### フォーマット

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Type 一覧

| Type       | 説明                                   |
| ---------- | -------------------------------------- |
| `feat`     | 新機能の追加                           |
| `fix`      | バグ修正                               |
| `docs`     | ドキュメントの変更                     |
| `style`    | コードスタイルの変更（フォーマット等） |
| `refactor` | リファクタリング                       |
| `perf`     | パフォーマンス改善                     |
| `test`     | テストの追加・修正                     |
| `chore`    | ビルドプロセス、補助ツールの変更       |

### Scope 一覧

| Scope          | 説明                       |
| -------------- | -------------------------- |
| `frontend`     | フロントエンド（src/）     |
| `backend`      | バックエンド（src-tauri/） |
| `database`     | データベース関連           |
| `auth`         | 認証関連                   |
| `github`       | GitHub API 関連            |
| `gamification` | ゲーミフィケーション機能   |
| `mock-server`  | モックサーバー             |
| `tools`        | CLI ツール（tools/）       |
| `docs`         | ドキュメント               |

### 例

```bash
git commit -m "feat(gamification): Add XP notification animation"
git commit -m "fix(auth): Fix token refresh logic"
git commit -m "docs: Update README with setup instructions"
git commit -m "refactor(database): Simplify migration structure"
```

---

## プルリクエスト

### PR の作成手順

1. 最新の`main`ブランチから分岐

   ```bash
   git checkout main
   git pull upstream main
   git checkout -b feature/issue-XXX-description
   ```

2. 変更を実装

3. テストを実行

   ```bash
   cd src-tauri && cargo test
   cargo check --package development-tools-ui
   ```

4. コミット＆プッシュ

   ```bash
   git push origin feature/issue-XXX-description
   ```

5. GitHub で PR を作成

### PR テンプレート

```markdown
## 概要

[変更内容の簡潔な説明]

## 関連 Issue

- Closes #XXX

## 変更内容

- [変更 1]
- [変更 2]

## テスト

- [ ] 単体テストを追加/更新
- [ ] 手動テスト完了
- [ ] `cargo test` パス
- [ ] `cargo check` パス

## スクリーンショット（UI 変更の場合）

[スクリーンショットを添付]

## チェックリスト

- [ ] コードは規約に従っている
- [ ] 必要なドキュメントを更新した
- [ ] セルフレビューを完了した
```

---

## コーディング規約

### Rust

- `rustfmt`でフォーマット
- `clippy`の警告を解消
- 関数・構造体には doc コメントを記載

```rust
/// ユーザーのXPを追加し、レベルアップイベントを発行
///
/// # Arguments
///
/// * `user_id` - 対象ユーザーのID
/// * `xp_amount` - 追加するXP量
///
/// # Returns
///
/// 更新後のレベル情報
pub async fn add_xp(user_id: i64, xp_amount: i64) -> Result<LevelInfo> {
    // ...
}
```

### コメントスタイル

- 日本語コメントを推奨
- TODO コメントには担当者と日付を記載

```rust
// TODO(sugai, 2025-11-30): レート制限の実装
// FIXME: エラーハンドリングを改善する必要あり
// NOTE: この処理は非同期で実行される
```

### ファイル構成（Legible Architecture）

プロジェクトは[AGENTS.md](AGENTS.md)に記載の Legible Architecture に従います：

- **Concept**: 独立した機能単位（他の Concept を直接参照しない）
- **Synchronization**: Concept 間の連携ルール

---

## ドキュメント更新

### 更新が必要なケース

1. **新機能の追加** - README.md、関連する PRD の更新
2. **API の変更** - API 仕様書の更新
3. **データベース変更** - スキーマドキュメントの更新
4. **バグ修正** - Issue 解決後に`resolved/`へ移動

### ドキュメント構造

```
docs/
├── requirements.md       # 全体要件定義
├── DEVELOPMENT.md        # 開発ガイド
├── ARCHITECTURE.md       # アーキテクチャ
├── prd/                  # PRD（製品要件定義）
├── 01_issues/            # Issue管理
│   ├── open/             # 未解決
│   └── resolved/         # 解決済み
├── 03_plans/             # 実装計画
└── 05_logs/              # 作業ログ
```

### 作業ログの記録

大きな実装作業後は作業ログを記録してください：

```
docs/05_logs/YYYY_MM/YYYYMMDD/XX_description.md
```

---

## 質問・サポート

質問がある場合は、GitHub Issues で質問してください。ラベル`question`を付けてください。

---

ご協力ありがとうございます！ 🎉
