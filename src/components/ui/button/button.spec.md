# Button Component Specification

## Related Files

- Implementation: `src/components/ui/button/mod.rs`
- Button: `src/components/ui/button/button.rs`

## Related Documentation

- Parent UI Module: `src/components/ui/mod.rs`

## Requirements

### 責務

Button コンポーネントモジュールは以下の責務を担当する：

1. **Button** - 汎用ボタンコンポーネント

   - 6 つのバリアント（Primary, Secondary, Ghost, Danger, Success, Outline）をサポート
   - 3 つのサイズ（Small, Medium, Large）をサポート
   - disabled 状態のサポート
   - full_width オプション
   - アクセシビリティ対応（focus ring, disabled state）

2. **IconButton** - アイコン専用ボタン
   - コンパクトなアイコンボタン
   - aria-label によるアクセシビリティ対応
   - 同様の 6 バリアントをサポート

### 状態構造

#### ButtonVariant

| Variant   | 用途                       | 背景色                       |
| --------- | -------------------------- | ---------------------------- |
| Primary   | メインアクション           | グラデーション (cyan→purple) |
| Secondary | セカンダリアクション       | bg-gm-bg-secondary + border  |
| Ghost     | 目立たないアクション       | transparent → hover で bg    |
| Danger    | 削除・破壊的アクション     | red 系                       |
| Success   | 成功・ポジティブアクション | green 系                     |
| Outline   | アウトラインボタン         | transparent + cyan border    |

#### ButtonSize

| Size   | Padding     | Text Size | Gap     |
| ------ | ----------- | --------- | ------- |
| Small  | px-3 py-1.5 | text-sm   | gap-1.5 |
| Medium | px-4 py-2   | text-base | gap-2   |
| Large  | px-6 py-3   | text-lg   | gap-2.5 |

### 公開 API

```rust
pub use button::{Button, ButtonVariant, ButtonSize, IconButton};
```

### スタイリング仕様

- 角丸: `rounded-2xl`（全バリアント共通）
- トランジション: `transition-all duration-200`
- フォーカスリング: `focus:ring-2 focus:ring-offset-2`
- disabled: `opacity-50 cursor-not-allowed pointer-events-none`

## Test Cases

### TC-001: Button 基本動作

- **Given**: Button が on_click ハンドラと共に初期化
- **When**: ボタンがクリックされる
- **Then**: on_click コールバックが 1 回実行される

### TC-002: Button disabled 状態

- **Given**: Button が disabled=true で初期化
- **When**: ボタンがクリックされる
- **Then**: on_click コールバックは実行されない

### TC-003: ButtonVariant スタイル適用

- **Given**: 各バリアントの Button が初期化
- **When**: レンダリングされる
- **Then**: 対応するスタイルクラスが適用される

### TC-004: IconButton アクセシビリティ

- **Given**: IconButton が label="Delete" で初期化
- **When**: レンダリングされる
- **Then**: aria-label="Delete" と title="Delete" が設定される
