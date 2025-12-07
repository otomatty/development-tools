# Input Component Specification (Solid.js)

## Related Files

- Implementation: `src/components/ui/form/Input.tsx` (includes Input, TextArea, and LabeledInput)
- Types: `src/types/ui.ts`
- Original (Leptos): `src/components/ui/form/input.rs`

## Related Documentation

- Issue: https://github.com/otomatty/development-tools/issues/136
- Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
- Original Spec: `src/components/ui/form/form.spec.md`

## Requirements

### 責務

Input コンポーネントモジュールは以下の責務を担当する：

1. **Input** - 汎用テキスト入力コンポーネント

   - テキスト、数値、パスワードなど複数の input type をサポート
   - placeholder 対応
   - disabled 状態のサポート
   - 3 つのサイズ（Small, Medium, Large）をサポート
   - 動的 ID サポート

2. **TextArea** - 複数行入力コンポーネント
   - 複数行テキスト入力
   - リサイズ可能/不可能のオプション
   - placeholder 対応
   - disabled 状態のサポート

3. **LabeledInput** - ラベル付き入力コンポーネント
   - 一貫したラベルスタイリング
   - 説明文（description）のサポート
   - 必須マーク（\*）のサポート
   - 一意な ID 自動生成（同一ラベルでも重複しない）
   - ラベルクリックで入力フィールドにフォーカス（アクセシビリティ対応）

### 状態構造

#### InputType

| Type      | HTML type | 用途           |
| --------- | --------- | -------------- |
| text      | text      | 通常のテキスト |
| password  | password  | パスワード     |
| number    | number    | 数値           |
| email     | email     | メールアドレス |
| url       | url       | URL            |
| search    | search    | 検索           |

#### InputSize

| Size   | Padding     | Text Size |
| ------ | ----------- | --------- |
| sm     | px-2 py-1   | text-sm   |
| md     | px-3 py-2   | text-base |
| lg     | px-4 py-3   | text-lg   |

### 公開 API

```typescript
export { Input, TextArea, LabeledInput } from './Input';
export type {
  InputProps,
  TextAreaProps,
  LabeledInputProps,
  InputType,
  InputSize,
} from '../../types/ui';
```

### スタイリング仕様

- 基本クラス: `w-full bg-gm-bg-secondary border border-gm-border rounded-md`
- フォーカス: `focus:outline-none focus:ring-2 focus:ring-gm-accent-cyan/50 focus:border-gm-accent-cyan`
- disabled: `opacity-50 cursor-not-allowed`
- トランジション: `transition-colors duration-200`

## Test Cases

### TC-001: Input 基本レンダリング

- **Given**: Input が value と onInput ハンドラ付きで初期化
- **When**: テキストが入力される
- **Then**: onInput コールバックが呼び出され、value が更新される

### TC-002: Input placeholder 表示

- **Given**: placeholder="Enter text..." が指定
- **When**: Input がレンダリング
- **Then**: placeholder 属性が正しく設定される

### TC-003: Input disabled 状態

- **Given**: disabled=true が指定
- **When**: Input がレンダリング
- **Then**: disabled 属性が設定され、スタイルが適用される

### TC-004: InputType スタイル適用

- **Given**: 各 inputType の Input が初期化
- **When**: レンダリングされる
- **Then**: 対応する type 属性が設定される

### TC-005: InputSize スタイル適用

- **Given**: 各サイズの Input が初期化
- **When**: レンダリングされる
- **Then**: 対応するサイズクラスが適用される

### TC-006: TextArea 基本動作

- **Given**: TextArea が value と onInput ハンドラ付きで初期化
- **When**: テキストが入力される
- **Then**: onInput コールバックが呼び出され、value が更新される

### TC-007: TextArea resizable オプション

- **Given**: resizable=false が指定
- **When**: TextArea がレンダリング
- **Then**: resize-none クラスが適用される

### TC-008: LabeledInput ラベル表示

- **Given**: label="Username" が指定
- **When**: LabeledInput がレンダリング
- **Then**: ラベルテキストが表示される

### TC-009: LabeledInput 必須マーク

- **Given**: required=true が指定
- **When**: LabeledInput がレンダリング
- **Then**: ラベルの横に "\*" が赤色で表示される

### TC-010: LabeledInput 一意 ID 生成

- **Given**: 同じ label="Email" を持つ 2 つの LabeledInput
- **When**: 両方がレンダリング
- **Then**: 異なる id 属性が生成される（重複しない）

### TC-011: LabeledInput 説明文表示

- **Given**: description="Enter your email address" が指定
- **When**: LabeledInput がレンダリング
- **Then**: 説明文が表示される

