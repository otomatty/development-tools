# Alert Components Specification

## Related Files

- Implementation: `src/components/ui/alert/mod.rs`
- Alert: `src/components/ui/alert/alert.rs`

## Related Documentation

- Parent UI Module: `src/components/ui/mod.rs`

## Requirements

### 責務

Alert コンポーネントモジュールは以下の責務を担当する：

1. **Alert** - アラートボックス

   - 4 つのバリアント（Success, Warning, Error, Info）をサポート
   - オプションのタイトル
   - 閉じるボタン（dismissible）
   - アイコン自動選択

2. **Banner** - バナー通知
   - フル width のバナー表示
   - 同じ 4 バリアントをサポート
   - 閉じるボタン（dismissible）

### 状態構造

#### AlertVariant

| Variant | 用途             | カラー         | アイコン       |
| ------- | ---------------- | -------------- | -------------- |
| Success | 成功メッセージ   | green-500      | check-circle   |
| Warning | 警告メッセージ   | amber-500      | alert-triangle |
| Error   | エラーメッセージ | red-500        | x-circle       |
| Info    | 情報メッセージ   | gm-accent-cyan | info           |

#### Alert Props

```rust
pub struct AlertProps {
    pub variant: AlertVariant,     // default: Error
    pub message: Signal<String>,
    pub title: Option<&'static str>,
    pub dismissible: bool,         // default: false
    pub on_dismiss: Option<F>,
    pub class: &'static str,
}
```

### 公開 API

```rust
pub use alert::{Alert, AlertVariant, Banner};
```

### スタイリング仕様

- 角丸: `rounded-2xl`
- パディング: `p-4`
- レイアウト: `flex items-start gap-3`
- 背景: 各カラーの `/20` 透過度
- ボーダー: 各カラーの `/50` 透過度

## Test Cases

### TC-001: Alert 基本表示

- **Given**: Alert が message="Error occurred" で初期化
- **When**: レンダリングされる
- **Then**: エラーメッセージとアイコンが表示される

### TC-002: Alert タイトル表示

- **Given**: Alert が title="Error" で初期化
- **When**: レンダリングされる
- **Then**: タイトルとメッセージが表示される

### TC-003: Alert dismissible

- **Given**: Alert が dismissible=true, on_dismiss で初期化
- **When**: 閉じるボタンがクリックされる
- **Then**: on_dismiss コールバックが実行される

### TC-004: AlertVariant アイコン選択

- **Given**: 各バリアントの Alert が初期化
- **When**: レンダリングされる
- **Then**: 対応するアイコンが表示される

### TC-005: Banner 基本表示

- **Given**: Banner が message で初期化
- **When**: レンダリングされる
- **Then**: フル width のバナーが表示される
