# Layout Components Specification

## Related Files

- Implementation: `src/components/ui/layout/mod.rs`
- PageHeader: `src/components/ui/layout/page_header.rs`
- EmptyState: `src/components/ui/layout/empty_state.rs`

## Related Documentation

- Parent UI Module: `src/components/ui/mod.rs`

## Requirements

### 責務

Layout コンポーネントモジュールは以下の責務を担当する：

1. **PageHeader** - ページヘッダー

   - タイトル表示（必須）
   - サブタイトル表示（オプション）
   - アクションエリア（ボタンなど）のサポート
   - 一貫したスタイリング

2. **PageHeaderAction** - 動的タイトル対応ページヘッダー

   - String 型のタイトル・サブタイトルを受け付ける
   - PageHeader と同様の機能

3. **EmptyState** - 空状態表示
   - アイコン表示
   - タイトル・説明文
   - アクションボタンエリア
   - 中央揃えレイアウト

### 状態構造

#### PageHeader Props

```rust
pub struct PageHeaderProps {
    pub title: &'static str,
    pub subtitle: Option<&'static str>,
    pub children: Option<Children>,  // アクションエリア
    pub class: &'static str,
}
```

#### EmptyState Props

```rust
pub struct EmptyStateProps {
    pub icon: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub children: Option<Children>,  // アクションボタン
    pub class: &'static str,
}
```

### 公開 API

```rust
pub use page_header::{PageHeader, PageHeaderAction};
pub use empty_state::EmptyState;
```

### スタイリング仕様

#### PageHeader

- タイトル: `text-2xl font-bold text-dt-text font-gaming`
- サブタイトル: `text-dt-text-sub mt-1`
- レイアウト: `flex items-center justify-between mb-6`

#### EmptyState

- コンテナ: `flex flex-col items-center justify-center py-12 text-center`
- アイコン背景: `bg-gm-bg-card/80 rounded-2xl border border-slate-700/50`
- アイコンサイズ: `w-12 h-12`
- タイトル: `text-lg font-semibold text-dt-text`
- 説明: `text-dt-text-sub text-sm max-w-md`

## Test Cases

### TC-001: PageHeader 基本表示

- **Given**: PageHeader が title="Projects" で初期化
- **When**: レンダリングされる
- **Then**: h1 タグに "Projects" が表示される

### TC-002: PageHeader サブタイトル表示

- **Given**: PageHeader が subtitle="Description" で初期化
- **When**: レンダリングされる
- **Then**: サブタイトルが p タグで表示される

### TC-003: PageHeader アクションエリア

- **Given**: PageHeader に children（ボタン）が渡される
- **When**: レンダリングされる
- **Then**: アクションエリアにボタンが表示される

### TC-004: EmptyState 基本表示

- **Given**: EmptyState が icon, title, description で初期化
- **When**: レンダリングされる
- **Then**: アイコン、タイトル、説明文が中央揃えで表示される

### TC-005: EmptyState アクションボタン

- **Given**: EmptyState に children（ボタン）が渡される
- **When**: レンダリングされる
- **Then**: 説明文の下にボタンが表示される
