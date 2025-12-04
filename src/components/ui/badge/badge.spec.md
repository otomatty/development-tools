# Badge Component Specification

## Related Files

- Implementation: `src/components/ui/badge/mod.rs`
- Badge: `src/components/ui/badge/badge.rs`

## Related Documentation

- Parent UI Module: `src/components/ui/mod.rs`

## Requirements

### 責務

Badge コンポーネントモジュールは以下の責務を担当する：

1. **Badge** - 汎用バッジコンポーネント

   - 7 つのバリアント（Success, Warning, Error, Info, Neutral, Purple, Gold）をサポート
   - 3 つのサイズ（Small, Medium, Large）をサポート
   - ステータスドット表示オプション

2. **DynamicBadge** - 動的テキスト対応バッジ

   - String 型のテキストを受け付ける
   - Badge と同様の機能

3. **StatusBadge** - ステータス専用バッジ
   - 事前定義されたステータス（Linked, Active, Pending など）
   - 自動的に適切なバリアントとテキストを適用
   - 常にドット表示

### 状態構造

#### BadgeVariant

| Variant | 用途                     | カラー           |
| ------- | ------------------------ | ---------------- |
| Success | 成功・アクティブ状態     | green-500        |
| Warning | 警告・保留状態           | amber-500        |
| Error   | エラー・失敗状態         | red-500          |
| Info    | 情報表示                 | gm-accent-cyan   |
| Neutral | デフォルト・非アクティブ | slate-500        |
| Purple  | 特別・プレミアム状態     | gm-accent-purple |
| Gold    | 実績・特別ステータス     | gm-accent-gold   |

#### BadgeSize

| Size   | Padding     | Text Size | Dot Size    |
| ------ | ----------- | --------- | ----------- |
| Small  | px-2 py-0.5 | text-xs   | w-1.5 h-1.5 |
| Medium | px-2.5 py-1 | text-sm   | w-2 h-2     |
| Large  | px-3 py-1.5 | text-base | w-2.5 h-2.5 |

#### Status（StatusBadge 用）

| Status    | Text         | Variant |
| --------- | ------------ | ------- |
| Linked    | "Linked"     | Success |
| NotLinked | "Not linked" | Warning |
| Active    | "Active"     | Success |
| Inactive  | "Inactive"   | Neutral |
| Pending   | "Pending"    | Warning |
| Error     | "Error"      | Error   |
| Success   | "Success"    | Success |
| Warning   | "Warning"    | Warning |
| Syncing   | "Syncing"    | Info    |
| Offline   | "Offline"    | Neutral |
| Online    | "Online"     | Success |

### 公開 API

```rust
pub use badge::{Badge, BadgeVariant, BadgeSize, DynamicBadge, StatusBadge, Status};
```

### スタイリング仕様

- 角丸: `rounded-2xl`（全サイズ共通）
- ボーダー: `border`（全バリアント共通）
- 背景: 各カラーの `/20` 透過度
- ボーダー: 各カラーの `/50` 透過度
- テキスト: 各カラーの `400` シェード

## Test Cases

### TC-001: Badge 基本表示

- **Given**: Badge が text="Active" で初期化
- **When**: レンダリングされる
- **Then**: "Active" テキストが表示される

### TC-002: Badge with_dot 表示

- **Given**: Badge が with_dot=true で初期化
- **When**: レンダリングされる
- **Then**: テキストの前にドットが表示される

### TC-003: BadgeVariant スタイル適用

- **Given**: 各バリアントの Badge が初期化
- **When**: レンダリングされる
- **Then**: 対応するカラークラスが適用される

### TC-004: StatusBadge 自動マッピング

- **Given**: StatusBadge が status=Status::Linked で初期化
- **When**: レンダリングされる
- **Then**: text="Linked", variant=Success, with_dot=true で表示される

### TC-005: BadgeSize サイズ適用

- **Given**: Badge が size=BadgeSize::Large で初期化
- **When**: レンダリングされる
- **Then**: px-3 py-1.5 text-base クラスが適用される
