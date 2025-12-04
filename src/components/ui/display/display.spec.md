# Display Components Specification

## Related Files

- Implementation: `src/components/ui/display/mod.rs`
- Avatar: `src/components/ui/display/avatar.rs`
- ProgressBar: `src/components/ui/display/progress_bar.rs`

## Related Documentation

- Parent UI Module: `src/components/ui/mod.rs`

## Requirements

### 責務

Display コンポーネントモジュールは以下の責務を担当する：

1. **Avatar** - アバター表示

   - 5 つのサイズ（XSmall, Small, Medium, Large, XLarge）をサポート
   - 画像 URL またはフォールバック（絵文字/イニシャル）表示
   - オプションのバッジ（レベル表示など）
   - ボーダーオプション

2. **ProgressBar** - 進捗バー
   - 6 つのカラーバリアント
   - ラベル表示（inside, outside, top）
   - アニメーション対応
   - カスタマイズ可能な高さ

### 状態構造

#### AvatarSize

| Size   | Dimensions | Text     |
| ------ | ---------- | -------- |
| XSmall | w-6 h-6    | text-xs  |
| Small  | w-8 h-8    | text-sm  |
| Medium | w-12 h-12  | text-lg  |
| Large  | w-16 h-16  | text-xl  |
| XLarge | w-20 h-20  | text-2xl |

#### ProgressBarVariant

| Variant | カラー                   |
| ------- | ------------------------ |
| Default | gradient (cyan → purple) |
| Cyan    | gm-accent-cyan           |
| Purple  | gm-accent-purple         |
| Gold    | gm-accent-gold           |
| Success | green-500                |
| Danger  | red-500                  |

### 公開 API

```rust
pub use avatar::{Avatar, AvatarSize};
pub use progress_bar::{ProgressBar, ProgressBarVariant};
```

### スタイリング仕様

#### Avatar

- 角丸: `rounded-2xl`
- ボーダー（有効時）: `border-2 border-gm-accent-cyan shadow-neon-cyan`
- フォールバック背景: `bg-gm-bg-secondary`

#### ProgressBar

- トラック: `bg-gm-bg-secondary rounded-full`
- バー: `rounded-full`
- アニメーション: `transition-all duration-500 ease-out`
- デフォルト高さ: `h-3`

## Test Cases

### TC-001: Avatar 画像表示

- **Given**: Avatar が src="url" で初期化
- **When**: レンダリングされる
- **Then**: img タグで画像が表示される

### TC-002: Avatar フォールバック表示

- **Given**: Avatar が src=None で初期化
- **When**: レンダリングされる
- **Then**: フォールバック（絵文字）が表示される

### TC-003: Avatar バッジ表示

- **Given**: Avatar に children（バッジ）が渡される
- **When**: レンダリングされる
- **Then**: バッジが右下に表示される

### TC-004: AvatarSize サイズ適用

- **Given**: Avatar が size=AvatarSize::Large で初期化
- **When**: レンダリングされる
- **Then**: w-16 h-16 クラスが適用される

### TC-005: ProgressBar 基本表示

- **Given**: ProgressBar が progress=50.0 で初期化
- **When**: レンダリングされる
- **Then**: バーが 50% の幅で表示される

### TC-006: ProgressBar ラベル表示

- **Given**: ProgressBar が show_label=true で初期化
- **When**: レンダリングされる
- **Then**: パーセンテージラベルが表示される

### TC-007: ProgressBar クランプ

- **Given**: ProgressBar が progress=150.0 で初期化
- **When**: レンダリングされる
- **Then**: バーは 100% に制限される

### TC-008: ProgressBarVariant カラー適用

- **Given**: 各バリアントの ProgressBar が初期化
- **When**: レンダリングされる
- **Then**: 対応するカラークラスが適用される
