# XP History Page Specification

## Related Files

- Implementation: `src/components/home/xp_history.rs`
- Tests: (統合テストとして実施)

## Related Documentation

- Types: `src/types/gamification.rs` - `XpHistoryEntry`, `XpBreakdown`
- API: `src/tauri_api.rs` - `get_xp_history`
- Backend:
  - `src-tauri/src/commands/gamification.rs` - `get_xp_history`
  - `src-tauri/src/database/repository/xp_history.rs` - `record_xp_gain`, `get_recent_xp_history`
  - `src-tauri/src/database/models/xp.rs` - `XpHistoryEntry`, `XpBreakdown`
  - `src-tauri/src/database/migrations.rs` - version 8 (breakdown_json column)

## Requirements

### 責務

- XP 取得履歴の一覧表示
- アクションタイプに応じたアイコン・色分け表示
- 相対時間表示（今日、昨日、○ 日前）
- アコーディオン形式での詳細情報表示
- XP 内訳（breakdown）データの表示

### 状態構造

- `xp_history: Vec<XpHistoryEntry>` - XP 履歴データ
- `loading: bool` - ローディング状態
- `error: Option<String>` - エラー状態

### XpHistoryEntry 構造

```rust
pub struct XpHistoryEntry {
    pub id: i64,
    pub user_id: i64,
    pub action_type: String,
    pub xp_amount: i32,
    pub description: Option<String>,
    pub github_event_id: Option<String>,
    pub breakdown: Option<XpBreakdown>,  // 追加
    pub created_at: String,
}
```

### XpBreakdown 構造（breakdown フィールド）

```rust
pub struct XpBreakdown {
    pub commits_xp: i32,
    pub prs_created_xp: i32,
    pub prs_merged_xp: i32,
    pub issues_created_xp: i32,
    pub issues_closed_xp: i32,
    pub reviews_xp: i32,
    pub stars_xp: i32,
    pub streak_bonus_xp: i32,
    pub total_xp: i32,
}
```

### コンポーネント

- `XpHistoryPage` - メインページコンポーネント
- `XpHistoryItem` - 履歴アイテムコンポーネント（アコーディオン対応）

### UI 仕様

- デフォルト表示件数: 20 件
- アクションタイプ別アイコン:
  - commit: 📝
  - pull_request: 🔀
  - pull_request_merged: ✅
  - review: 👀
  - issue: 📋
  - issue_closed: ✔️
  - streak_bonus: 🔥
  - star: ⭐
- 時間表示: 今日/昨日/○ 日前
- アコーディオン詳細:
  - アクションタイプ
  - 獲得 XP
  - 取得日時（絶対時間）
  - 履歴 ID
  - XP 計算内訳（breakdown データがある場合）
  - 内訳がない場合は XP 単価参考表示（過去データ用フォールバック）

### データベース

- `xp_history` テーブルに `breakdown_json` カラムを追加（migration version 8）
- `github_sync` 時に XpBreakdown を JSON で保存
- 過去のデータは breakdown が null（フォールバック表示）

## Test Cases

### TC-001: 初期ロード

- Given: ユーザーがログイン済み
- When: XP 履歴ページを表示
- Then: 最新 20 件の XP 履歴が表示される

### TC-002: ローディング表示

- Given: データ取得中
- When: ページ表示
- Then: ローディングスケルトンが表示される

### TC-003: 空状態

- Given: XP 履歴が 0 件
- When: ページ表示
- Then: 「まだ履歴がありません」メッセージが表示される

### TC-004: エラー状態

- Given: API 呼び出しが失敗
- When: ページ表示
- Then: エラーメッセージが表示される

### TC-005: アコーディオン展開

- Given: XP 履歴が表示されている
- When: 履歴アイテムをクリック
- Then: 詳細情報がアコーディオン形式で展開される

### TC-006: XP 内訳表示（breakdown あり）

- Given: breakdown データを持つ XP 履歴が表示されている
- When: アコーディオンを展開
- Then: 実際の XP 内訳が表示される（コミット +XX XP 等）

### TC-007: XP 内訳表示（breakdown なし）

- Given: breakdown データを持たない過去の XP 履歴が表示されている
- When: アコーディオンを展開
- Then: XP 単価参考情報がフォールバック表示される
