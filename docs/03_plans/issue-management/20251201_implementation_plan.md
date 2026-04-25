# 実装計画: GitHub Issue 管理機能（Linear 風カンバン）

**作成日**: 2025-12-01  
**関連 Issue**: [#59](https://github.com/otomatty/development-tools/issues/59)  
**ステータス**: 計画中

---

## 1. 概要

Tauri アプリ内で GitHub Issue を Linear 風のカンバン UI で管理できる機能を実装する。
GitHub 上の操作（push, PR 作成, PR マージ）によって自動的にステータスが更新される仕組みを構築する。

### 基本原則

| 原則                              | 説明                                                                    |
| --------------------------------- | ----------------------------------------------------------------------- |
| **Single Source of Truth**        | データの実体は GitHub（Issues, Labels）に置く。ローカル DB はキャッシュ |
| **1 プロジェクト = 1 リポジトリ** | シンプルな構造で管理                                                    |
| **自動ステータス同期**            | GitHub Actions によるラベル自動更新                                     |
| **ブランチ命名規則強制**          | `type/issue番号-description` 形式                                       |

---

## 2. ブランチ命名規則

### 2.1 フォーマット

```
<type>/<issue-number>-<description>
```

### 2.2 許可される type

| Type       | 用途             | 例                          |
| ---------- | ---------------- | --------------------------- |
| `feat`     | 新機能           | `feat/123-add-login`        |
| `fix`      | バグ修正         | `fix/456-fix-crash`         |
| `docs`     | ドキュメント     | `docs/789-update-readme`    |
| `refactor` | リファクタリング | `refactor/101-cleanup-code` |
| `test`     | テスト           | `test/102-add-unit-tests`   |
| `chore`    | その他           | `chore/103-update-deps`     |

### 2.3 正規表現パターン

```regex
^(feat|fix|docs|refactor|test|chore)\/(\d+)-[\w-]+$
```

---

## 3. データモデル

### 3.1 新規テーブル

```sql
-- プロジェクト（1プロジェクト = 1リポジトリ）
CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    -- リポジトリ情報（1:1）
    github_repo_id INTEGER,                    -- GitHub のリポジトリID
    repo_owner TEXT,                           -- リポジトリオーナー
    repo_name TEXT,                            -- リポジトリ名
    repo_full_name TEXT,                       -- "owner/repo"
    is_actions_setup BOOLEAN DEFAULT FALSE,    -- GitHub Actions 設定済みか
    last_synced_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, github_repo_id)
);

-- Issueキャッシュ
CREATE TABLE cached_issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,               -- projects.id
    github_issue_id INTEGER NOT NULL,          -- GitHub の Issue ID
    number INTEGER NOT NULL,                   -- Issue 番号
    title TEXT NOT NULL,
    body TEXT,
    state TEXT NOT NULL DEFAULT 'open',        -- open/closed
    status TEXT NOT NULL DEFAULT 'backlog',    -- backlog/todo/in-progress/in-review/done/cancelled
    priority TEXT,                             -- high/medium/low/null
    assignee_login TEXT,
    assignee_avatar_url TEXT,
    labels_json TEXT,                          -- JSON配列
    html_url TEXT,                             -- GitHub上のURL
    github_created_at DATETIME,
    github_updated_at DATETIME,
    cached_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    UNIQUE(project_id, github_issue_id)
);

-- インデックス
CREATE INDEX idx_projects_user ON projects(user_id);
CREATE INDEX idx_cached_issues_project ON cached_issues(project_id);
CREATE INDEX idx_cached_issues_status ON cached_issues(project_id, status);
```

### 3.2 ステータス定義

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IssueStatus {
    Backlog,
    Todo,
    InProgress,
    InReview,
    Done,
    Cancelled,
}
```

---

## 4. ファイル構成

### 4.1 バックエンド

```
src-tauri/src/
├── commands/
│   └── issues.rs              # 🆕 Issue管理コマンド
├── database/
│   ├── migrations.rs          # ✏️ マイグレーション追加
│   └── models/
│       ├── mod.rs             # ✏️ モジュール追加
│       └── issue.rs           # 🆕 Issue/Project モデル
├── github/
│   ├── mod.rs                 # ✏️ モジュール追加
│   ├── issues.rs              # 🆕 Issue API クライアント
│   └── actions_template.rs    # 🆕 GitHub Actions テンプレート
└── lib.rs                     # ✏️ コマンド登録
```

### 4.2 フロントエンド

```
src/
├── components/
│   ├── mod.rs                 # ✏️ モジュール追加
│   ├── sidebar.rs             # ✏️ Projects メニュー追加
│   └── issues/                # 🆕
│       ├── mod.rs
│       ├── projects_page.rs       # プロジェクト一覧/作成
│       ├── project_dashboard.rs   # カンバンダッシュボード
│       ├── kanban_board.rs        # カンバンボード
│       ├── kanban_column.rs       # ステータスカラム
│       ├── issue_card.rs          # Issueカード
│       ├── issue_detail_modal.rs  # Issue詳細モーダル
│       ├── project_settings.rs    # プロジェクト設定
│       └── link_repository.rs     # リポジトリリンクUI
├── types/
│   ├── mod.rs                 # ✏️ モジュール追加
│   └── issue.rs               # 🆕 フロントエンド用型
└── tauri_api.rs               # ✏️ API追加
```

---

## 5. 実装フェーズ

### Phase 1: データベース・基盤（1 日）

| タスク | ファイル                    | 内容                                     |
| ------ | --------------------------- | ---------------------------------------- |
| P1-01  | `database/migrations.rs`    | projects, cached_issues テーブル追加     |
| P1-02  | `database/models/issue.rs`  | Project, CachedIssue, IssueStatus モデル |
| P1-03  | `database/repository.rs`    | CRUD メソッド追加                        |
| P1-04  | `types/issue.rs` (frontend) | フロントエンド用型定義                   |

### Phase 2: GitHub API 拡張（1 日）

| タスク | ファイル                     | 内容                             |
| ------ | ---------------------------- | -------------------------------- |
| P2-01  | `github/issues.rs`           | Issue 取得、ラベル操作 API       |
| P2-02  | `github/issues.rs`           | ステータスラベル作成機能         |
| P2-03  | `github/actions_template.rs` | GitHub Actions YAML テンプレート |
| P2-04  | `github/issues.rs`           | PR 作成機能（Actions 設定用）    |

### Phase 3: Tauri コマンド（1 日）

| タスク | ファイル             | 内容                                         |
| ------ | -------------------- | -------------------------------------------- |
| P3-01  | `commands/issues.rs` | create_project, get_projects, delete_project |
| P3-02  | `commands/issues.rs` | link_repository, setup_github_actions        |
| P3-03  | `commands/issues.rs` | sync_issues, get_project_issues              |
| P3-04  | `commands/issues.rs` | update_issue_status, create_issue            |
| P3-05  | `lib.rs`             | コマンド登録                                 |

### Phase 4: フロントエンド UI（3-4 日）

| タスク | ファイル                      | 内容                      |
| ------ | ----------------------------- | ------------------------- |
| P4-01  | `sidebar.rs`                  | Projects メニュー追加     |
| P4-02  | `app.rs`                      | ルーティング追加          |
| P4-03  | `issues/projects_page.rs`     | プロジェクト一覧・作成 UI |
| P4-04  | `issues/project_dashboard.rs` | ダッシュボード全体        |
| P4-05  | `issues/kanban_board.rs`      | カンバンボード            |
| P4-06  | `issues/kanban_column.rs`     | ステータスカラム          |
| P4-07  | `issues/issue_card.rs`        | Issue カード              |
| P4-08  | `issues/link_repository.rs`   | リポジトリリンク UI       |

### Phase 5: 詳細機能（2 日）

| タスク | ファイル                       | 内容                      |
| ------ | ------------------------------ | ------------------------- |
| P5-01  | `issues/issue_detail_modal.rs` | Issue 詳細表示            |
| P5-02  | `issues/project_settings.rs`   | プロジェクト設定          |
| P5-03  | ドラッグ&ドロップ              | ステータス変更の D&D 実装 |
| P5-04  | 同期機能                       | 定期同期・手動同期        |

### Phase 6: テスト・調整（1 日）

| タスク | 内容                      |
| ------ | ------------------------- |
| P6-01  | 統合テスト                |
| P6-02  | エラーハンドリング確認    |
| P6-03  | UI 調整・レスポンシブ対応 |

---

## 6. API 仕様

### 6.1 Tauri コマンド

```rust
// プロジェクト管理
#[tauri::command]
async fn create_project(name: String, description: Option<String>) -> Result<Project, String>;

#[tauri::command]
async fn get_projects() -> Result<Vec<Project>, String>;

#[tauri::command]
async fn get_project(project_id: i64) -> Result<Project, String>;

#[tauri::command]
async fn update_project(project_id: i64, name: String, description: Option<String>) -> Result<Project, String>;

#[tauri::command]
async fn delete_project(project_id: i64) -> Result<(), String>;

// リポジトリリンク
#[tauri::command]
async fn get_user_repositories() -> Result<Vec<Repository>, String>;

#[tauri::command]
async fn link_repository(project_id: i64, owner: String, repo: String) -> Result<Project, String>;

#[tauri::command]
async fn setup_github_actions(project_id: i64) -> Result<String, String>; // PR URL を返す

// Issue 管理
#[tauri::command]
async fn sync_project_issues(project_id: i64) -> Result<Vec<CachedIssue>, String>;

#[tauri::command]
async fn get_project_issues(project_id: i64, status: Option<IssueStatus>) -> Result<Vec<CachedIssue>, String>;

#[tauri::command]
async fn update_issue_status(project_id: i64, issue_number: i32, status: IssueStatus) -> Result<CachedIssue, String>;

#[tauri::command]
async fn create_issue(project_id: i64, title: String, body: Option<String>, status: IssueStatus) -> Result<CachedIssue, String>;
```

---

## 7. UI 設計

### 7.1 サイドバー（更新後）

```
┌───────────────────┐
│ 🏠 Dashboard      │
│ 📋 Projects       │  ← 新規追加
│ 🔧 Tools          │
│ ⚙️ Settings       │
└───────────────────┘
```

### 7.2 Projects ページ

```
┌─────────────────────────────────────────────────────────────────┐
│  📋 Projects                                      [+ New Project]│
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ 📁 My Awesome App                                   [→]     ││
│  │    otomatty/my-awesome-app                                  ││
│  │    Last synced: 5 minutes ago  │  12 open issues            ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ 📁 Another Project                                  [→]     ││
│  │    otomatty/another-project                                 ││
│  │    Last synced: 1 hour ago  │  5 open issues                ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ ➕ Create New Project                                        ││
│  │    Link a GitHub repository to start tracking issues         ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 7.3 Project Dashboard（カンバン）

```
┌─────────────────────────────────────────────────────────────────┐
│  ← Back │ 📁 My Awesome App                    [⚙️] [🔄 Sync]  │
│         │ otomatty/my-awesome-app                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐    │
│ │ Backlog │ │  Todo   │ │In Progr.│ │In Review│ │  Done   │    │
│ │   (3)   │ │   (2)   │ │   (1)   │ │   (1)   │ │   (5)   │    │
│ ├─────────┤ ├─────────┤ ├─────────┤ ├─────────┤ ├─────────┤    │
│ │┌───────┐│ │┌───────┐│ │┌───────┐│ │┌───────┐│ │┌───────┐│    │
│ ││ #12   ││ ││ #15   ││ ││ #18   ││ ││ #20   ││ ││ #10   ││    │
│ ││Fix bug││ ││Add... ││ ││Impl...││ ││Update ││ ││Close..││    │
│ ││[🔴P1] ││ ││[🟡P2] ││ ││       ││ ││       ││ ││       ││    │
│ │└───────┘│ │└───────┘│ │└───────┘│ │└───────┘│ │└───────┘│    │
│ │┌───────┐│ │┌───────┐│ │         │ │         │ │┌───────┐│    │
│ ││ #14   ││ ││ #16   ││ │         │ │         │ ││ #11   ││    │
│ ││...    ││ ││...    ││ │         │ │         │ ││...    ││    │
│ │└───────┘│ │└───────┘│ │         │ │         │ │└───────┘│    │
│ │┌───────┐│ │         │ │         │ │         │ │...      │    │
│ ││ #17   ││ │         │ │         │ │         │ │         │    │
│ ││...    ││ │         │ │         │ │         │ │         │    │
│ │└───────┘│ │         │ │         │ │         │ │         │    │
│ └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 8. GitHub Actions テンプレート

リポジトリリンク時に自動生成される `.github/workflows/issue-status-sync.yml`:

```yaml
name: Issue Status Sync

on:
  push:
    branches-ignore:
      - main
      - master
  pull_request:
    types: [opened, closed, reopened]

jobs:
  update-status:
    runs-on: ubuntu-latest
    steps:
      - name: Validate Branch Name & Extract Issue Number
        id: extract
        run: |
          BRANCH="${{ github.head_ref || github.ref_name }}"
          # ブランチ命名規則: type/issue番号-description
          if [[ ! "$BRANCH" =~ ^(feat|fix|docs|refactor|test|chore)/([0-9]+)-[a-zA-Z0-9_-]+$ ]]; then
            echo "Branch name does not match required pattern: type/<issue-number>-<description>"
            echo "Examples: feat/123-add-login, fix/456-fix-crash"
            echo "issue_number=" >> $GITHUB_OUTPUT
            exit 0
          fi
          ISSUE_NUMBER="${BASH_REMATCH[2]}"
          echo "issue_number=$ISSUE_NUMBER" >> $GITHUB_OUTPUT
          echo "Branch: $BRANCH, Issue: $ISSUE_NUMBER"

      - name: Skip if no issue number
        if: steps.extract.outputs.issue_number == ''
        run: echo "No valid issue number found, skipping status update"

      - name: Update Status on Push (In Progress)
        if: github.event_name == 'push' && steps.extract.outputs.issue_number != ''
        uses: actions/github-script@v7
        with:
          script: |
            const issueNumber = parseInt('${{ steps.extract.outputs.issue_number }}');
            const statusLabels = ['status:backlog', 'status:todo', 'status:in-progress', 'status:in-review', 'status:done', 'status:cancelled'];

            try {
              const { data: issue } = await github.rest.issues.get({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber
              });
              
              // 既に done または cancelled の場合はスキップ
              const currentStatus = issue.labels.find(l => statusLabels.includes(l.name));
              if (currentStatus && (currentStatus.name === 'status:done' || currentStatus.name === 'status:cancelled')) {
                console.log(`Issue #${issueNumber} is already ${currentStatus.name}, skipping`);
                return;
              }
              
              // status:xxx ラベルを削除
              for (const label of issue.labels) {
                if (statusLabels.includes(label.name)) {
                  await github.rest.issues.removeLabel({
                    owner: context.repo.owner,
                    repo: context.repo.repo,
                    issue_number: issueNumber,
                    name: label.name
                  }).catch(() => {});
                }
              }
              
              // status:in-progress を追加
              await github.rest.issues.addLabels({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber,
                labels: ['status:in-progress']
              });
              
              console.log(`Updated issue #${issueNumber} to in-progress`);
            } catch (error) {
              console.log(`Failed to update issue #${issueNumber}: ${error.message}`);
            }

      - name: Update Status on PR Open (In Review)
        if: github.event_name == 'pull_request' && github.event.action == 'opened' && steps.extract.outputs.issue_number != ''
        uses: actions/github-script@v7
        with:
          script: |
            const issueNumber = parseInt('${{ steps.extract.outputs.issue_number }}');
            const statusLabels = ['status:backlog', 'status:todo', 'status:in-progress', 'status:in-review', 'status:done', 'status:cancelled'];

            try {
              const { data: issue } = await github.rest.issues.get({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber
              });
              
              for (const label of issue.labels) {
                if (statusLabels.includes(label.name)) {
                  await github.rest.issues.removeLabel({
                    owner: context.repo.owner,
                    repo: context.repo.repo,
                    issue_number: issueNumber,
                    name: label.name
                  }).catch(() => {});
                }
              }
              
              await github.rest.issues.addLabels({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber,
                labels: ['status:in-review']
              });
              
              console.log(`Updated issue #${issueNumber} to in-review`);
            } catch (error) {
              console.log(`Failed to update issue #${issueNumber}: ${error.message}`);
            }

      - name: Update Status on PR Merge (Done)
        if: github.event_name == 'pull_request' && github.event.action == 'closed' && github.event.pull_request.merged == true && steps.extract.outputs.issue_number != ''
        uses: actions/github-script@v7
        with:
          script: |
            const issueNumber = parseInt('${{ steps.extract.outputs.issue_number }}');
            const statusLabels = ['status:backlog', 'status:todo', 'status:in-progress', 'status:in-review', 'status:done', 'status:cancelled'];

            try {
              const { data: issue } = await github.rest.issues.get({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber
              });
              
              for (const label of issue.labels) {
                if (statusLabels.includes(label.name)) {
                  await github.rest.issues.removeLabel({
                    owner: context.repo.owner,
                    repo: context.repo.repo,
                    issue_number: issueNumber,
                    name: label.name
                  }).catch(() => {});
                }
              }
              
              await github.rest.issues.addLabels({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber,
                labels: ['status:done']
              });
              
              // Issue をクローズ
              await github.rest.issues.update({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber,
                state: 'closed',
                state_reason: 'completed'
              });
              
              console.log(`Updated issue #${issueNumber} to done and closed`);
            } catch (error) {
              console.log(`Failed to update issue #${issueNumber}: ${error.message}`);
            }

      - name: Revert Status on PR Close without Merge
        if: github.event_name == 'pull_request' && github.event.action == 'closed' && github.event.pull_request.merged == false && steps.extract.outputs.issue_number != ''
        uses: actions/github-script@v7
        with:
          script: |
            const issueNumber = parseInt('${{ steps.extract.outputs.issue_number }}');
            const statusLabels = ['status:backlog', 'status:todo', 'status:in-progress', 'status:in-review', 'status:done', 'status:cancelled'];

            try {
              const { data: issue } = await github.rest.issues.get({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber
              });
              
              for (const label of issue.labels) {
                if (statusLabels.includes(label.name)) {
                  await github.rest.issues.removeLabel({
                    owner: context.repo.owner,
                    repo: context.repo.repo,
                    issue_number: issueNumber,
                    name: label.name
                  }).catch(() => {});
                }
              }
              
              // PR Close の場合は in-progress に戻す
              await github.rest.issues.addLabels({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: issueNumber,
                labels: ['status:in-progress']
              });
              
              console.log(`Reverted issue #${issueNumber} to in-progress (PR closed without merge)`);
            } catch (error) {
              console.log(`Failed to revert issue #${issueNumber}: ${error.message}`);
            }
```

---

## 9. ステータスラベル

リポジトリリンク時に自動作成されるラベル:

| ラベル名             | 色                 | 説明       |
| -------------------- | ------------------ | ---------- |
| `status:backlog`     | `#E2E2E2` (グレー) | バックログ |
| `status:todo`        | `#0052CC` (青)     | 予定       |
| `status:in-progress` | `#FBCA04` (黄)     | 作業中     |
| `status:in-review`   | `#7C3AED` (紫)     | レビュー中 |
| `status:done`        | `#0E8A16` (緑)     | 完了       |
| `status:cancelled`   | `#6A737D` (グレー) | キャンセル |
| `priority:high`      | `#D73A4A` (赤)     | 高優先度   |
| `priority:medium`    | `#FBCA04` (黄)     | 中優先度   |
| `priority:low`       | `#0E8A16` (緑)     | 低優先度   |

---

## 10. 工数見積もり

| フェーズ | 内容               | 見積もり    |
| -------- | ------------------ | ----------- |
| Phase 1  | データベース・基盤 | 1 日        |
| Phase 2  | GitHub API 拡張    | 1 日        |
| Phase 3  | Tauri コマンド     | 1 日        |
| Phase 4  | フロントエンド UI  | 3-4 日      |
| Phase 5  | 詳細機能           | 2 日        |
| Phase 6  | テスト・調整       | 1 日        |
| **合計** |                    | **9-10 日** |

---

## 11. 将来の拡張（スコープ外）

- Webhook によるリアルタイム更新
- 複数リポジトリの横断管理
- カスタムステータスの追加
- ゲーミフィケーション連携（Issue Close で XP 付与）
- Issue テンプレート管理
- マイルストーン表示
