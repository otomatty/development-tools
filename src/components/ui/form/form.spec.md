# Form Components Specification

## Related Files

- Implementation: `src/components/ui/form/mod.rs`
- ToggleSwitch: `src/components/ui/form/toggle_switch.rs`
- OptionForm: `src/components/ui/form/option_form.rs`
- Input: `src/components/ui/form/input.rs` (新規)
  - `Input` - 基本入力コンポーネント
  - `LabeledInput` - ラベル付き入力コンポーネント
  - `Textarea` - 複数行入力コンポーネント

## Related Documentation

- Parent Issue: [#115](https://github.com/otomatty/development-tools/issues/115) Phase 2 フォーム・フィードバックコンポーネントの移動
- Parent UI Module: `src/components/ui/mod.rs`

## Requirements

### 責務

フォームコンポーネントモジュールは以下の責務を担当する：

1. **ToggleSwitch** - ON/OFF 切り替えスイッチ

   - 3 つのサイズ（Small, Medium, Large）をサポート
   - アニメーション付きの視覚的フィードバック
   - disabled 状態のサポート
   - アクセシビリティ対応（role="switch", aria-checked）

2. **OptionForm** - ツールオプション入力フォーム

   - 複数のオプションタイプをサポート（Boolean, Select, Path, Number, String）
   - ファイル/フォルダ選択ダイアログ連携
   - バリデーションサポート

3. **Input** (新規) - 汎用テキスト入力

   - テキスト、数値、パスワードなど複数の input type をサポート
   - placeholder 対応
   - disabled 状態のサポート
   - 動的 ID サポート（`Option<String>`）

4. **LabeledInput** (新規) - ラベル付き入力コンポーネント
   - 一貫したラベルスタイリング
   - 説明文（description）のサポート
   - 必須マーク（\*）のサポート
   - 一意な ID 自動生成（同一ラベルでも重複しない）
   - ラベルクリックで入力フィールドにフォーカス（アクセシビリティ対応）

### 状態構造

#### ToggleSwitch Props

```rust
pub struct ToggleSwitchProps {
    pub enabled: bool,
    pub on_toggle: impl Fn() + 'static,
    pub label_id: Option<&'static str>,
    pub size: ToggleSwitchSize,
    pub disabled: bool,
}

pub enum ToggleSwitchSize {
    Small,  // w-10 h-5
    Medium, // w-12 h-6 (default)
    Large,  // w-14 h-7
}
```

#### Input Props (新規)

```rust
pub struct InputProps {
    pub value: RwSignal<String>,
    pub input_type: &'static str,       // default: "text"
    pub placeholder: Option<&'static str>,
    pub disabled: bool,                  // default: false
    pub class: Option<&'static str>,
}
```

### 公開 API

```rust
// mod.rs からの re-export
pub use toggle_switch::{ToggleSwitch, ToggleSwitchSize};
pub use option_form::OptionForm;
pub use input::{Input, LabeledInput, Textarea, InputType, InputSize};
```

## Test Cases

### TC-001: ToggleSwitch 基本動作

- **Given**: ToggleSwitch が enabled=false で初期化
- **When**: on_toggle コールバックが呼び出される
- **Then**: コールバック関数が 1 回実行される

### TC-002: ToggleSwitch サイズ変更

- **Given**: ToggleSwitchSize::Small が指定
- **When**: コンポーネントがレンダリング
- **Then**: ボタンに "w-10 h-5" クラスが適用される

### TC-003: ToggleSwitch disabled 状態

- **Given**: ToggleSwitch が disabled=true で初期化
- **When**: コンポーネントがレンダリング
- **Then**: "opacity-50 cursor-not-allowed" クラスが適用される

### TC-004: Input 基本レンダリング

- **Given**: Input が value シグナル付きで初期化
- **When**: テキストが入力される
- **Then**: value シグナルが更新される

### TC-005: Input placeholder 表示

- **Given**: placeholder="Enter text..." が指定
- **When**: Input がレンダリング
- **Then**: placeholder 属性が正しく設定される

### TC-006: LabeledInput ラベル表示

- **Given**: label="Username" が指定
- **When**: LabeledInput がレンダリング
- **Then**: ラベルテキストが表示される

### TC-007: LabeledInput 必須マーク

- **Given**: required=true が指定
- **When**: LabeledInput がレンダリング
- **Then**: ラベルの横に "\*" が赤色で表示される

### TC-008: LabeledInput 一意 ID 生成

- **Given**: 同じ label="Email" を持つ 2 つの LabeledInput
- **When**: 両方がレンダリング
- **Then**: 異なる id 属性が生成される（重複しない）

### TC-009: OptionForm 複数オプション表示

- **Given**: 3 つの ToolOption が渡される
- **When**: OptionForm がレンダリング
- **Then**: 3 つの OptionField がレンダリングされる

### TC-010: モジュールインポート確認

- **Given**: `use crate::components::ui::form::ToggleSwitch;`
- **When**: コードがコンパイル
- **Then**: ToggleSwitch が正しくインポートできる

## Implementation Notes

- 移動元: `settings/toggle_switch.rs`, `settings/toast.rs`, `option_form.rs`
- 後方互換性のため、旧パスからの re-export を一時的に維持
- AGENTS.md の Concept 独立性原則に従い、他の Concept を直接参照しない
