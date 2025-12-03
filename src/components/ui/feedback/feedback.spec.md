# Feedback Components Specification

## Related Files

- Implementation: `src/components/ui/feedback/mod.rs`
- Toast: `src/components/ui/feedback/toast.rs`
- Loading: `src/components/ui/feedback/loading.rs` (新規)
- Notification: `src/components/ui/feedback/notification.rs` (新規、オプション)

## Related Documentation

- Parent Issue: [#115](https://github.com/otomatty/development-tools/issues/115) Phase 2 フォーム・フィードバックコンポーネントの移動
- Parent UI Module: `src/components/ui/mod.rs`

## Requirements

### 責務

フィードバックコンポーネントモジュールは以下の責務を担当する：

1. **Toast** - 一時的な通知表示

   - 4 つのタイプ（Success, Error, Info, Warning）をサポート
   - 自動非表示（時間経過）
   - 固定位置表示（画面右下）
   - アクセシビリティ対応（role="alert", aria-live="polite"）

2. **InlineToast** - インライン通知表示

   - パネル内での通知表示用
   - 同じ 4 タイプをサポート
   - 相対位置での表示

3. **Loading** (新規) - ローディング表示

   - 3 つのサイズ（small, medium, large）をサポート
   - オプションのテキスト表示
   - アニメーション付きスピナー

4. **Notification** (新規、オプション) - 永続的な通知
   - 閉じるボタン付き
   - アクション付き通知

### 状態構造

#### Toast Props

```rust
pub struct ToastProps {
    pub visible: ReadSignal<bool>,
    pub message: Signal<String>,
    pub toast_type: ToastType,  // default: ToastType::Info
}

pub enum ToastType {
    Success,
    Error,
    Info,
    Warning,
}
```

#### Loading Props (新規)

```rust
pub struct LoadingProps {
    pub size: &'static str,      // "small" | "medium" | "large", default: "medium"
    pub text: Option<&'static str>,
}
```

### 公開 API

```rust
// mod.rs からの re-export
pub use toast::{Toast, InlineToast, ToastType};
pub use loading::Loading;
```

### スタイリング仕様

#### Toast タイプ別スタイル

| Type    | Background           | Border                   | Text                | Glow                 |
| ------- | -------------------- | ------------------------ | ------------------- | -------------------- |
| Success | bg-green-900/90      | border-green-500/50      | text-green-200      | rgba(34,197,94,0.3)  |
| Error   | bg-red-900/90        | border-red-500/50        | text-red-200        | rgba(239,68,68,0.3)  |
| Info    | bg-gm-accent-cyan/20 | border-gm-accent-cyan/50 | text-gm-accent-cyan | rgba(6,182,212,0.3)  |
| Warning | bg-amber-900/90      | border-amber-500/50      | text-amber-200      | rgba(245,158,11,0.3) |

#### Loading サイズ別スタイル

| Size   | Dimensions |
| ------ | ---------- |
| small  | w-4 h-4    |
| medium | w-8 h-8    |
| large  | w-12 h-12  |

## Test Cases

### TC-001: Toast 表示/非表示

- **Given**: Toast が visible=true で初期化
- **When**: コンポーネントがレンダリング
- **Then**: Toast が画面に表示される

### TC-002: Toast 非表示状態

- **Given**: Toast が visible=false で初期化
- **When**: コンポーネントがレンダリング
- **Then**: Toast が画面に表示されない（<Show>で制御）

### TC-003: Toast Success タイプスタイル

- **Given**: ToastType::Success が指定
- **When**: Toast がレンダリング
- **Then**: 緑色のスタイル（bg-green-900/90）が適用される

### TC-004: Toast Error タイプスタイル

- **Given**: ToastType::Error が指定
- **When**: Toast がレンダリング
- **Then**: 赤色のスタイル（bg-red-900/90）が適用される

### TC-005: Toast メッセージ表示

- **Given**: message="保存しました" が指定
- **When**: Toast がレンダリング
- **Then**: "保存しました" テキストが表示される

### TC-006: InlineToast レンダリング

- **Given**: InlineToast が visible クロージャ付きで初期化
- **When**: visible() が true を返す
- **Then**: InlineToast が表示される

### TC-007: Loading 基本レンダリング

- **Given**: Loading が size="medium" で初期化
- **When**: コンポーネントがレンダリング
- **Then**: スピナーアニメーションが表示される

### TC-008: Loading テキスト表示

- **Given**: Loading が text="読み込み中..." で初期化
- **When**: コンポーネントがレンダリング
- **Then**: "読み込み中..." テキストがスピナーの下に表示される

### TC-009: Loading サイズ small

- **Given**: Loading が size="small" で初期化
- **When**: コンポーネントがレンダリング
- **Then**: "w-4 h-4" クラスが適用される

### TC-010: モジュールインポート確認

- **Given**: `use crate::components::ui::feedback::Toast;`
- **When**: コードがコンパイル
- **Then**: Toast が正しくインポートできる

## Implementation Notes

- 移動元: `settings/toast.rs`
- Toast は固定位置（fixed bottom-6 right-6 z-50）で表示
- InlineToast は相対位置で表示
- アニメーション: `animate-slideInUp` クラスを使用
- 後方互換性のため、旧パスからの re-export を一時的に維持
