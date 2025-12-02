# XP History Page Specification

## Related Files

- Implementation: `src/components/home/xp_history.rs`
- Tests: (統合テストとして実施)

## Related Documentation

- Types: `src/types/gamification.rs` - `XpHistoryEntry`
- API: `src/tauri_api.rs` - `get_xp_history`
- Backend: `src-tauri/src/commands/gamification.rs` - `get_xp_history`

## Requirements

### 責務

- XP 取得履歴の一覧表示
- アクションタイプに応じたアイコン・色分け表示
- 相対時間表示（今日、昨日、○ 日前）

### 状態構造

- `xp_history: Vec<XpHistoryEntry>` - XP 履歴データ
- `loading: bool` - ローディング状態
- `error: Option<String>` - エラー状態

### コンポーネント

- `XpHistoryPage` - メインページコンポーネント
- `XpHistoryItem` - 履歴アイテムコンポーネント

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
