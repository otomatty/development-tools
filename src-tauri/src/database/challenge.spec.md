# Challenge System Specification

## Related Files

### Backend
- Implementation: `src-tauri/src/database/challenge.rs`
- Repository: `src-tauri/src/database/repository.rs` (Challenge section)
- Commands: `src-tauri/src/commands/challenge.rs`
- Models: `src-tauri/src/database/models.rs` (Challenge struct)
- Migrations: `src-tauri/src/database/migrations.rs` (v3: start_stats)

### Frontend
- Types: `src/types.rs` (ChallengeInfo, CreateChallengeRequest)
- API: `src/tauri_api.rs` (challenge functions)
- UI Component: `src/components/home/challenge_card.rs`

### Documentation
- Issue: `docs/01_issues/open/2025_11/` (Issue #9)
- Logs: `docs/05_logs/2025_11/20251130/`

## Requirements

### 責務
チャレンジシステムは、ユーザーのGitHubアクティビティに基づいた目標設定と達成追跡を提供する。

### 機能概要

#### 1. チャレンジタイプ
- **デイリーチャレンジ**: 1日以内に達成する短期目標
- **ウィークリーチャレンジ**: 1週間以内に達成する中期目標

#### 2. ターゲットメトリクス
- `commits`: コミット数
- `prs`: プルリクエスト数
- `reviews`: コードレビュー数
- `issues`: イシュー数

#### 3. チャレンジステータス
- `active`: 進行中
- `completed`: 達成（目標値に到達）
- `failed`: 失敗（期限切れ）

### 状態構造

```rust
// チャレンジ情報
pub struct Challenge {
    pub id: i64,
    pub user_id: i64,
    pub challenge_type: String,    // "daily" | "weekly"
    pub target_metric: String,     // "commits" | "prs" | "reviews" | "issues"
    pub target_value: i32,         // 目標値
    pub current_value: i32,        // 現在の進捗
    pub reward_xp: i32,            // 達成時のXP報酬
    pub start_date: DateTime<Utc>, // 開始日時
    pub end_date: DateTime<Utc>,   // 終了日時
    pub status: String,            // "active" | "completed" | "failed"
    pub completed_at: Option<DateTime<Utc>>,
}

// 進捗追跡用の開始統計
pub struct ChallengeStats {
    pub commits: i32,
    pub prs: i32,
    pub reviews: i32,
    pub issues: i32,
}
```

### アクション

#### Repository Operations
| メソッド | 説明 |
|---------|------|
| `create_challenge` | 新しいチャレンジを作成 |
| `create_challenge_with_stats` | 開始統計付きでチャレンジを作成 |
| `get_challenge_by_id` | IDでチャレンジを取得 |
| `get_active_challenges` | アクティブなチャレンジ一覧を取得 |
| `get_all_challenges` | すべてのチャレンジを取得 |
| `update_challenge_progress` | 進捗を更新（ターゲット到達時は自動complete） |
| `complete_challenge` | チャレンジを完了状態にする |
| `fail_challenge` | チャレンジを失敗状態にする |
| `fail_expired_challenges` | 期限切れチャレンジを失敗にする |
| `delete_challenge` | チャレンジを削除 |
| `has_active_challenge` | 特定タイプ/メトリクスのアクティブチャレンジが存在するか |
| `get_challenge_completion_count` | 完了チャレンジ数を取得 |
| `get_consecutive_weekly_completions` | 連続週間チャレンジ完了数を取得 |
| `get_last_daily_challenge_date` | 最後の日次チャレンジ日を取得 |
| `get_last_weekly_challenge_date` | 最後の週次チャレンジ日を取得 |
| `get_challenge_start_stats` | チャレンジの開始統計を取得 |

#### Challenge Generation Functions
| 関数 | 説明 |
|------|------|
| `calculate_recommended_targets` | 履歴に基づいて推奨ターゲットを計算 |
| `generate_daily_challenges` | 日次チャレンジテンプレートを生成 |
| `generate_weekly_challenges` | 週次チャレンジテンプレートを生成 |
| `calculate_reward_xp` | メトリクスと目標値からXP報酬を計算 |
| `calculate_challenge_period` | チャレンジの開始・終了日時を計算 |
| `should_generate_daily_challenges` | 新しい日次チャレンジが必要か判定 |
| `should_generate_weekly_challenges` | 新しい週次チャレンジが必要か判定 |

#### Tauri Commands
| コマンド | 説明 |
|---------|------|
| `get_active_challenges` | アクティブなチャレンジ一覧を取得 |
| `get_all_challenges` | すべてのチャレンジを取得 |
| `create_challenge` | 新しいチャレンジを作成 |
| `delete_challenge` | チャレンジを削除 |
| `update_challenge_progress` | 進捗を更新 |
| `get_challenge_stats` | チャレンジ統計を取得 |

### XP報酬計算

| メトリクス | 1単位あたりXP |
|-----------|--------------|
| commits | 10 XP |
| prs | 40 XP |
| reviews | 20 XP |
| issues | 25 XP |

### 自動生成ロジック

GitHub同期時（`sync_github_stats`）に以下の処理が実行される：

1. **日次チャレンジ生成チェック**
   - 最後の日次チャレンジが今日より前なら新規生成
   - 現在のGitHub統計を開始統計として保存

2. **週次チャレンジ生成チェック**
   - 最後の週次チャレンジが今週の月曜日より前なら新規生成
   - 現在のGitHub統計を開始統計として保存

3. **進捗更新**
   - アクティブなチャレンジの進捗を計算
   - 進捗 = 現在の統計 - 開始統計
   - ターゲット到達時は自動的に完了

4. **期限切れチェック**
   - 期限が過ぎたアクティブチャレンジを失敗に変更

## Test Cases

### TC-001: create_challenge
- Given: ユーザーが存在
- When: create_challenge(user_id, "weekly", "commits", 10, 100, start, end)を実行
- Then: 新しいチャレンジが作成され、statusは"active"

### TC-002: create_challenge_with_stats
- Given: ユーザーが存在
- When: create_challenge_with_stats(..., start_stats_json)を実行
- Then: チャレンジが作成され、start_stats_jsonが保存される

### TC-003: update_challenge_progress
- Given: アクティブなチャレンジ（target_value=10）
- When: update_challenge_progress(id, 5)を実行
- Then: current_valueが5に更新

### TC-004: update_challenge_progress_exceeds_target
- Given: アクティブなチャレンジ（target_value=10）
- When: update_challenge_progress(id, 15)を実行
- Then: current_valueが10にキャップされ、statusが"completed"に変更

### TC-005: complete_challenge
- Given: アクティブなチャレンジ
- When: complete_challenge(id)を実行
- Then: statusが"completed"、completed_atが設定される

### TC-006: fail_expired_challenges
- Given: 期限切れのアクティブなチャレンジ
- When: fail_expired_challenges(user_id)を実行
- Then: 該当チャレンジのstatusが"failed"に変更

### TC-007: get_active_challenges
- Given: 複数のチャレンジ（active, completed, failed）
- When: get_active_challenges(user_id)を実行
- Then: status="active"のチャレンジのみ返される

### TC-008: should_generate_daily_challenges
- Given: 最後の日次チャレンジが昨日
- When: should_generate_daily_challenges(Some(yesterday), now)を実行
- Then: trueを返す

### TC-009: should_generate_weekly_challenges
- Given: 最後の週次チャレンジが先週
- When: should_generate_weekly_challenges(Some(last_week), now)を実行
- Then: trueを返す

### TC-010: calculate_reward_xp
- Given: メトリクス="commits", 目標値=5
- When: calculate_reward_xp("commits", 5)を実行
- Then: 50 (= 5 * 10)を返す

### TC-011: challenge_stats_serialization
- Given: ChallengeStats { commits: 100, prs: 20, reviews: 15, issues: 5 }
- When: JSON形式でシリアライズ/デシリアライズ
- Then: 元の値と一致

### TC-012: get_last_daily_challenge_date
- Given: 日次チャレンジが存在
- When: get_last_daily_challenge_date(user_id)を実行
- Then: 最後の日次チャレンジの開始日を返す

## UI仕様

### チャレンジカード
- 位置: ホーム画面、Stats Displayの隣
- 表示内容:
  - チャレンジタイプバッジ（デイリー/ウィークリー）
  - ターゲットメトリクスとアイコン
  - 進捗（現在値/目標値）
  - プログレスバー（シマーアニメーション付き）
  - 残り時間または完了/失敗ステータス
  - XP報酬

### カラーリング
- アクティブ（通常）: purple-cyan グラデーション
- アクティブ（75%以上）: gold-pink グラデーション
- 完了: success グリーン
- 失敗: error レッド
