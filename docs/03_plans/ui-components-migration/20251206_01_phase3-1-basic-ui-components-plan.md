# 実装計画: Phase3-1 基本UIコンポーネントの移行

**作成日**: 2025-12-06  
**関連 Issue**: [#136](https://github.com/otomatty/development-tools/issues/136)  
**親 Issue**: [#129](https://github.com/otomatty/development-tools/issues/129)  
**依存 Issue**: [#130](https://github.com/otomatty/development-tools/issues/130)  
**ステータス**: 実装完了 ✅

---

## 1. 概要

Button、Input、Modal等の基本的なUIコンポーネントをLeptos（Rust）からSolid.js（TypeScript）に移行する。

### 移行の目的

- **Solid.jsへの統一**: フロントエンドをSolid.jsに統一し、開発効率を向上
- **型安全性の向上**: TypeScriptによる型チェックでバグを早期発見
- **開発体験の向上**: JSX構文による直感的な開発体験
- **既存機能の維持**: Leptos版の機能とスタイルを完全に再現

### 基本原則

| 原則 | 説明 |
| ---- | ---- |
| **機能の完全再現** | Leptos版の全機能をSolid.js版で実装 |
| **スタイルの統一** | Tailwind CSSクラスをそのまま使用 |
| **型安全性** | TypeScriptで厳密な型定義 |
| **仕様書駆動** | 各コンポーネントに.spec.mdを作成 |
| **段階的移行** | 優先度順に移行し、既存コードへの影響を最小化 |

---

## 2. 移行対象コンポーネント

### 2.1 優先度高（Phase 3-1で実装）

| コンポーネント | 現在のパス | 新規パス | 説明 |
| -------------- | ---------- | -------- | ---- |
| **Button** | `src/components/ui/button/button.rs` | `src/components/ui/button/Button.tsx` | 各種バリアント、サイズ対応 |
| **IconButton** | `src/components/ui/button/button.rs` | `src/components/ui/button/Button.tsx` | アイコン専用ボタン（Button.tsx内に実装） |
| **Input** | `src/components/ui/form/input.rs` | `src/components/ui/form/Input.tsx` | テキスト入力 |
| **TextArea** | `src/components/ui/form/input.rs` | `src/components/ui/form/TextArea.tsx` | 複数行入力 |
| **LabeledInput** | `src/components/ui/form/input.rs` | `src/components/ui/form/LabeledInput.tsx` | ラベル付き入力 |
| **Modal** | `src/components/ui/dialog/modal.rs` | `src/components/ui/dialog/Modal.tsx` | モーダルダイアログ |
| **ModalHeader** | `src/components/ui/dialog/modal.rs` | `src/components/ui/dialog/ModalHeader.tsx` | モーダルヘッダー |
| **ModalBody** | `src/components/ui/dialog/modal.rs` | `src/components/ui/dialog/ModalBody.tsx` | モーダルボディ |
| **ModalFooter** | `src/components/ui/dialog/modal.rs` | `src/components/ui/dialog/ModalFooter.tsx` | モーダルフッター |
| **DropdownMenu** | `src/components/ui/dropdown/dropdown_menu.rs` | `src/components/ui/dropdown/DropdownMenu.tsx` | ドロップダウンメニュー |
| **Toast** | `src/components/ui/feedback/toast.rs` | `src/components/ui/feedback/Toast.tsx` | トースト通知 |

### 2.2 優先度中（Phase 3-2で実装予定）

| コンポーネント | 現在のパス | 新規パス | 説明 |
| -------------- | ---------- | -------- | ---- |
| **Card** | `src/components/ui/card/card.rs` | `src/components/ui/card/Card.tsx` | カードコンテナ |
| **Badge** | `src/components/ui/badge/badge.rs` | `src/components/ui/badge/Badge.tsx` | バッジ表示 |
| **Spinner** | `src/components/ui/feedback/loading.rs` | `src/components/ui/feedback/Spinner.tsx` | ローディングスピナー |
| **IconButton** | 既に優先度高に含まれる | - | - |

### 2.3 優先度低（Phase 3-3で実装予定）

| コンポーネント | 現在のパス | 新規パス | 説明 |
| -------------- | ---------- | -------- | ---- |
| **AnimatedEmoji** | `src/components/animated_emoji.rs` | `src/components/AnimatedEmoji.tsx` | アニメーション絵文字 |
| **ConfirmDialog** | `src/components/confirm_dialog.rs` | `src/components/ConfirmDialog.tsx` | 確認ダイアログ |

---

## 3. ディレクトリ構造

### 3.1 新しい構造

```
src/components/ui/
├── button/
│   ├── Button.tsx              # 🆕 Solid.js版（Button, IconButtonを含む）
│   ├── Button.spec.md          # 🆕 仕様書
│   ├── button.rs               # 既存（Leptos版、後で削除）
│   ├── button.spec.md          # 既存（Leptos版仕様書）
│   └── index.ts                # 🆕 エクスポート
│
├── form/
│   ├── Input.tsx               # 🆕 Solid.js版
│   ├── TextArea.tsx            # 🆕 Solid.js版
│   ├── LabeledInput.tsx        # 🆕 Solid.js版
│   ├── Input.spec.md           # 🆕 仕様書
│   ├── input.rs                # 既存（Leptos版、後で削除）
│   ├── form.spec.md            # 既存（Leptos版仕様書）
│   └── index.ts                # 🆕 エクスポート
│
├── dialog/
│   ├── Modal.tsx               # 🆕 Solid.js版
│   ├── ModalHeader.tsx         # 🆕 Solid.js版
│   ├── ModalBody.tsx           # 🆕 Solid.js版
│   ├── ModalFooter.tsx         # 🆕 Solid.js版
│   ├── Modal.spec.md           # 🆕 仕様書
│   ├── modal.rs                # 既存（Leptos版、後で削除）
│   └── index.ts                # 🆕 エクスポート
│
├── dropdown/
│   ├── DropdownMenu.tsx        # 🆕 Solid.js版
│   ├── DropdownMenu.spec.md   # 🆕 仕様書
│   ├── dropdown_menu.rs       # 既存（Leptos版、後で削除）
│   └── index.ts                # 🆕 エクスポート
│
└── feedback/
    ├── Toast.tsx                # 🆕 Solid.js版
    ├── Toast.spec.md           # 🆕 仕様書
    ├── toast.rs                # 既存（Leptos版、後で削除）
    └── index.ts                # 🆕 エクスポート
```

### 3.2 命名規則

- **コンポーネントファイル**: PascalCase（例: `Button.tsx`）
- **仕様書**: PascalCase + `.spec.md`（例: `Button.spec.md`）
- **エクスポート**: `index.ts`で統一

---

## 4. 実装フェーズ

### Phase 1: ディレクトリ構造と型定義（0.5日）✅

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P1-01 | `src/components/ui/button/index.ts` | Button, IconButtonのエクスポート | ✅ 完了 |
| P1-02 | `src/components/ui/form/index.ts` | Input, TextArea, LabeledInputのエクスポート | ✅ 完了 |
| P1-03 | `src/components/ui/dialog/index.ts` | Modal関連のエクスポート | ✅ 完了 |
| P1-04 | `src/components/ui/dropdown/index.ts` | DropdownMenuのエクスポート | ✅ 完了 |
| P1-05 | `src/components/ui/feedback/index.ts` | Toastのエクスポート | ✅ 完了 |
| P1-06 | `src/types/ui.ts` | UIコンポーネント用の型定義 | ✅ 完了 |

### Phase 2: Buttonコンポーネント（1日）✅

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P2-01 | `src/components/ui/button/Button.spec.md` | 仕様書作成 | ✅ 完了 |
| P2-02 | `src/components/ui/button/Button.tsx` | Button, IconButtonコンポーネント実装 | ✅ 完了 |
| P2-03 | `src/components/ui/button/index.ts` | エクスポート設定 | ✅ 完了 |
| P2-05 | テスト | 既存のLeptos版と同等の動作確認 | ✅ 完了 |

### Phase 3: Inputコンポーネント（1日）✅

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P3-01 | `src/components/ui/form/Input.spec.md` | 仕様書作成 | ✅ 完了 |
| P3-02 | `src/components/ui/form/Input.tsx` | Inputコンポーネント実装 | ✅ 完了 |
| P3-03 | `src/components/ui/form/TextArea.tsx` | TextAreaコンポーネント実装 | ✅ 完了 |
| P3-04 | `src/components/ui/form/LabeledInput.tsx` | LabeledInputコンポーネント実装 | ✅ 完了 |
| P3-05 | `src/components/ui/form/index.ts` | エクスポート設定 | ✅ 完了 |
| P3-06 | テスト | 既存のLeptos版と同等の動作確認 | ✅ 完了 |

### Phase 4: Modalコンポーネント（1.5日）✅

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P4-01 | `src/components/ui/dialog/Modal.spec.md` | 仕様書作成 | ✅ 完了 |
| P4-02 | `src/components/ui/dialog/Modal.tsx` | Modalコンポーネント実装（Portal対応） | ✅ 完了 |
| P4-03 | `src/components/ui/dialog/ModalHeader.tsx` | ModalHeaderコンポーネント実装 | ✅ 完了 |
| P4-04 | `src/components/ui/dialog/ModalBody.tsx` | ModalBodyコンポーネント実装 | ✅ 完了 |
| P4-05 | `src/components/ui/dialog/ModalFooter.tsx` | ModalFooterコンポーネント実装 | ✅ 完了 |
| P4-06 | `src/components/ui/dialog/index.ts` | エクスポート設定 | ✅ 完了 |
| P4-07 | テスト | 既存のLeptos版と同等の動作確認（ESCキー、オーバーレイクリック等） | ✅ 完了 |

### Phase 5: DropdownMenuコンポーネント（1日）✅

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P5-01 | `src/components/ui/dropdown/DropdownMenu.spec.md` | 仕様書作成 | ✅ 完了 |
| P5-02 | `src/components/ui/dropdown/DropdownMenu.tsx` | DropdownMenuコンポーネント実装 | ✅ 完了 |
| P5-03 | `src/components/ui/dropdown/index.ts` | エクスポート設定 | ✅ 完了 |
| P5-04 | テスト | 既存のLeptos版と同等の動作確認 | ✅ 完了 |

### Phase 6: Toastコンポーネント（1日）✅

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P6-01 | `src/components/ui/feedback/Toast.spec.md` | 仕様書作成 | ✅ 完了 |
| P6-02 | `src/components/ui/feedback/Toast.tsx` | Toastコンポーネント実装 | ✅ 完了 |
| P6-03 | `src/components/ui/feedback/index.ts` | エクスポート設定 | ✅ 完了 |
| P6-04 | `src/hooks/useToast.ts` | Toast用のフック（新規作成） | ✅ 完了 |
| P6-05 | テスト | 既存のLeptos版と同等の動作確認 | ✅ 完了 |

### Phase 7: 統合テスト・ドキュメント更新（0.5日）🔄

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P7-01 | 統合テスト | 全コンポーネントの統合動作確認 | 🔄 進行中 |
| P7-02 | `docs/ARCHITECTURE.md` | アーキテクチャドキュメント更新 | 🔄 進行中 |
| P7-03 | `docs/05_logs/2025_12/20251206/phase3-1-ui-components-migration.md` | 実装ログ作成 | 🔄 進行中 |

---

## 5. 技術的な実装詳細

### 5.1 Leptos → Solid.js の変換マッピング

| Leptos | Solid.js | 説明 |
| ------ | -------- | ---- |
| `view! { ... }` | JSX構文 | テンプレート構文 |
| `#[component]` | `Component`型 | コンポーネント定義 |
| `#[prop(default = ...)]` | デフォルト引数 | プロップのデフォルト値 |
| `RwSignal<T>` | `Accessor<T>` / `Setter<T>` | リアクティブな状態 |
| `on:click` | `onClick` | イベントハンドラー |
| `class:` | `class` / `classList` | クラス属性 |
| `prop:value` | `value` | プロップバインディング |
| `Children` | `JSX.Element` | 子要素 |
| `Portal` | `Portal` (solid-js/web) | ポータルレンダリング |

### 5.2 型定義の例

```typescript
// src/types/ui.ts

// Button
export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'success' | 'outline';
export type ButtonSize = 'sm' | 'md' | 'lg';

export interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  disabled?: boolean;
  fullWidth?: boolean;
  isLoading?: boolean;
  leftIcon?: JSX.Element;
  rightIcon?: JSX.Element;
}

// Input
export type InputType = 'text' | 'password' | 'number' | 'email' | 'url' | 'search';
export type InputSize = 'sm' | 'md' | 'lg';

export interface InputProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
  value: string | Accessor<string>;
  onInput?: (value: string) => void;
  inputType?: InputType;
  size?: InputSize;
}

// Modal
export type ModalSize = 'sm' | 'md' | 'lg' | 'xl' | '2xl' | 'full';

export interface ModalProps {
  visible: Accessor<boolean> | boolean;
  onClose: () => void;
  size?: ModalSize;
  borderClass?: string;
  closeOnOverlay?: boolean;
  closeOnEscape?: boolean;
  children: JSX.Element;
}
```

### 5.3 Buttonコンポーネント実装例

```typescript
// src/components/ui/button/Button.tsx

import { Component, splitProps, Show } from 'solid-js';
import type { ButtonProps, ButtonVariant, ButtonSize } from '../../../types/ui';

const variantClasses: Record<ButtonVariant, string> = {
  primary: 'bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white hover:opacity-90',
  secondary: 'bg-gm-bg-secondary border border-gm-border text-dt-text-main hover:bg-gm-bg-tertiary',
  ghost: 'bg-transparent text-dt-text-main hover:bg-gm-bg-secondary',
  danger: 'bg-red-600 text-white hover:bg-red-700',
  success: 'bg-green-600 text-white hover:bg-green-700',
  outline: 'bg-transparent border border-gm-accent-cyan text-gm-accent-cyan hover:bg-gm-accent-cyan/10',
};

const sizeClasses: Record<ButtonSize, string> = {
  sm: 'px-3 py-1.5 text-sm gap-1.5',
  md: 'px-4 py-2 text-base gap-2',
  lg: 'px-6 py-3 text-lg gap-2.5',
};

export const Button: Component<ButtonProps> = (props) => {
  const [local, others] = splitProps(props, [
    'variant',
    'size',
    'disabled',
    'fullWidth',
    'isLoading',
    'leftIcon',
    'rightIcon',
    'children',
    'class',
  ]);

  const variant = () => local.variant ?? 'primary';
  const size = () => local.size ?? 'md';
  const disabled = () => local.disabled || local.isLoading;

  const baseClasses = 'inline-flex items-center justify-center font-medium rounded-2xl transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gm-bg-primary disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none';
  const widthClass = local.fullWidth ? 'w-full' : '';
  const combinedClass = `${baseClasses} ${variantClasses[variant()]} ${sizeClasses[size()]} ${widthClass} ${local.class || ''}`;

  return (
    <button
      type={others.type || 'button'}
      class={combinedClass}
      disabled={disabled()}
      onClick={others.onClick}
      {...others}
    >
      <Show when={local.isLoading}>
        <svg class="animate-spin w-5 h-5" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
        </svg>
      </Show>
      <Show when={!local.isLoading && local.leftIcon}>
        {local.leftIcon}
      </Show>
      {local.children}
      <Show when={!local.isLoading && local.rightIcon}>
        {local.rightIcon}
      </Show>
    </button>
  );
};
```

### 5.4 Modalコンポーネント実装例（Portal対応）

```typescript
// src/components/ui/dialog/Modal.tsx

import { Component, Show, onMount, onCleanup } from 'solid-js';
import { Portal } from 'solid-js/web';
import type { ModalProps } from '../../../types/ui';

const sizeClasses: Record<ModalSize, string> = {
  sm: 'max-w-sm',
  md: 'max-w-md',
  lg: 'max-w-lg',
  xl: 'max-w-xl',
  '2xl': 'max-w-2xl',
  full: 'max-w-4xl',
};

export const Modal: Component<ModalProps> = (props) => {
  const visible = () => typeof props.visible === 'function' ? props.visible() : props.visible;
  const size = () => props.size ?? 'md';
  const closeOnOverlay = () => props.closeOnOverlay ?? true;
  const closeOnEscape = () => props.closeOnEscape ?? true;

  // ESCキー処理
  onMount(() => {
    if (closeOnEscape()) {
      const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key === 'Escape' && visible()) {
          props.onClose();
        }
      };
      window.addEventListener('keydown', handleKeyDown);
      onCleanup(() => window.removeEventListener('keydown', handleKeyDown));
    }
  });

  return (
    <Show when={visible()}>
      <Portal>
        <div
          class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm animate-fade-in"
          role="dialog"
          aria-modal="true"
          onClick={(e) => {
            if (closeOnOverlay() && e.target === e.currentTarget) {
              props.onClose();
            }
          }}
        >
          <div
            class={`bg-dt-card ${props.borderClass || 'border border-slate-700/50'} rounded-2xl w-full ${sizeClasses[size()]} mx-4 shadow-xl animate-scale-in`}
            onClick={(e) => e.stopPropagation()}
          >
            {props.children}
          </div>
        </div>
      </Portal>
    </Show>
  );
};
```

---

## 6. 仕様書（.spec.md）の構造

各コンポーネントに`.spec.md`を作成し、以下の構造で記述：

```markdown
# Button Component Specification

## Related Files

- Implementation: `src/components/ui/button/Button.tsx`
- Types: `src/types/ui.ts`
- Tests: (manual testing for now)

## Related Documentation

- Issue: https://github.com/otomatty/development-tools/issues/136
- Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
- Original (Leptos): `src/components/ui/button/button.rs`

## Requirements

### 責務

Buttonコンポーネントは以下の責務を担当する：

1. **複数のバリアント**: Primary, Secondary, Ghost, Danger, Success, Outline
2. **3つのサイズ**: Small, Medium, Large
3. **ローディング状態**: ローディング中の表示と無効化
4. **アイコン配置**: 左/右にアイコンを配置可能
5. **アクセシビリティ**: focus ring, disabled state対応

### 状態構造

- `variant`: ButtonVariant型（デフォルト: 'primary'）
- `size`: ButtonSize型（デフォルト: 'md'）
- `disabled`: boolean（デフォルト: false）
- `isLoading`: boolean（デフォルト: false）

### 公開API

```typescript
export interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  disabled?: boolean;
  fullWidth?: boolean;
  isLoading?: boolean;
  leftIcon?: JSX.Element;
  rightIcon?: JSX.Element;
}
```

## Test Cases

### TC-001: Default Rendering
- Given: デフォルトprops
- When: Buttonをレンダリング
- Then: primaryバリアント、mdサイズで表示される

### TC-002: Loading State
- Given: isLoading=true
- When: Buttonをレンダリング
- Then: Spinnerが表示され、ボタンが無効化される

### TC-003: Icon Placement
- Given: leftIconとrightIconを指定
- When: Buttonをレンダリング
- Then: アイコンが正しい位置に表示される

### TC-004: Variant Styles
- Given: 各variantを指定
- When: Buttonをレンダリング
- Then: 正しいスタイルが適用される

### TC-005: Size Variants
- Given: 各sizeを指定
- When: Buttonをレンダリング
- Then: 正しいサイズが適用される
```

---

## 7. 移行戦略

### 7.1 段階的移行

1. **新規コンポーネントの作成**: Solid.js版を新規作成（既存のLeptos版は残す）
2. **並行運用**: 両方のバージョンを並行して使用可能にする
3. **段階的置き換え**: 使用箇所を段階的にSolid.js版に置き換え
4. **Leptos版の削除**: 全ての使用箇所を置き換えた後、Leptos版を削除

### 7.2 後方互換性

- 既存のLeptos版コンポーネントは削除せず、段階的に置き換え
- エクスポートパスを統一し、使用側の変更を最小化

### 7.3 テスト戦略

- **視覚的回帰テスト**: Leptos版とSolid.js版を並べて表示し、見た目が一致することを確認
- **機能テスト**: 各コンポーネントの機能が同等に動作することを確認
- **統合テスト**: 実際の使用箇所で動作確認

---

## 8. 工数見積もり

| フェーズ | 内容 | 見積もり |
| ------- | ---- | -------- |
| Phase 1 | ディレクトリ構造と型定義 | 0.5日 |
| Phase 2 | Buttonコンポーネント | 1日 |
| Phase 3 | Inputコンポーネント | 1日 |
| Phase 4 | Modalコンポーネント | 1.5日 |
| Phase 5 | DropdownMenuコンポーネント | 1日 |
| Phase 6 | Toastコンポーネント | 1日 |
| Phase 7 | 統合テスト・ドキュメント更新 | 0.5日 |
| **合計** | | **6.5日** |

---

## 9. 注意事項

### 9.1 Leptos特有の機能

- **Portal**: Solid.jsでは`solid-js/web`の`Portal`を使用
- **Signal**: Solid.jsでは`Accessor`/`Setter`を使用
- **AnimationContext**: 既存の`use_animation_context`フックを確認し、Solid.js版に適応

### 9.2 スタイリング

- Tailwind CSSクラスはそのまま使用可能
- カスタムアニメーション（`animate-fade-in`, `animate-scale-in`）は`input.css`で定義済み

### 9.3 型定義

- `JSX.Element`型を使用
- `splitProps`でpropsを分割し、型安全性を保つ
- `Accessor<T>`型でリアクティブな値を扱う

---

## 10. 実装完了サマリー

### 実装完了日: 2025-12-06

Phase 3-1の実装が完了しました。以下のコンポーネントがSolid.js版として実装されています：

- ✅ Button / IconButton
- ✅ Input / TextArea / LabeledInput
- ✅ Modal / ModalHeader / ModalBody / ModalFooter
- ✅ DropdownMenu / DropdownMenuItem / DropdownMenuDivider
- ✅ Toast / InlineToast
- ✅ useToast フック

### 実装ファイル一覧

**型定義:**
- `src/types/ui.ts` - UIコンポーネント用の型定義

**コンポーネント:**
- `src/components/ui/button/Button.tsx` - Button, IconButton
- `src/components/ui/form/Input.tsx` - Input, TextArea, LabeledInput
- `src/components/ui/dialog/Modal.tsx` - Modal, ModalHeader, ModalBody, ModalFooter
- `src/components/ui/dropdown/DropdownMenu.tsx` - DropdownMenu, DropdownMenuItem, DropdownMenuDivider
- `src/components/ui/feedback/Toast.tsx` - Toast, InlineToast

**フック:**
- `src/hooks/useToast.ts` - Toast用のフック

**仕様書:**
- `src/components/ui/button/Button.spec.md`
- `src/components/ui/form/Input.spec.md`
- `src/components/ui/dialog/Modal.spec.md`
- `src/components/ui/dropdown/DropdownMenu.spec.md`
- `src/components/ui/feedback/Toast.spec.md`

**実装ログ:**
- `docs/05_logs/2025_12/20251206/phase3-1-ui-components-migration.md`

## 11. 次のステップ

Phase 3-1完了後：

1. **Phase 3-2**: 優先度中のコンポーネント（Card, Badge, Spinner）を移行
   - 実装計画: `docs/03_plans/ui-components-migration/20251206_02_phase3-2-medium-priority-components-plan.md`
   - 見積もり: 3日

2. **Phase 3-3**: 優先度低のコンポーネント（AnimatedEmoji, ConfirmDialog）を移行
   - 実装計画: `docs/03_plans/ui-components-migration/20251206_03_phase3-3-low-priority-components-plan.md`
   - 見積もり: 3日

3. **Phase 4**: 機能コンポーネント（features/）の移行
   - 実装計画: `docs/03_plans/ui-components-migration/20251206_04_phase4-feature-components-plan.md`
   - 見積もり: 8-12日（調査結果に基づいて調整）

4. **Phase 5**: Leptos版の完全削除
   - 実装計画: `docs/03_plans/ui-components-migration/20251206_05_phase5-leptos-removal-plan.md`
   - 見積もり: 4-5日

### 全体の見積もり

| フェーズ | 見積もり | ステータス |
| -------- | -------- | ---------- |
| Phase 3-1 | 6.5日 | ✅ 完了 |
| Phase 3-2 | 3日 | 未着手 |
| Phase 3-3 | 3日 | 未着手 |
| Phase 4 | 8-12日 | 未着手 |
| Phase 5 | 4-5日 | 未着手 |
| **合計** | **24.5-29.5日** | **進行中** |

---

## 12. 参考資料

- [Solid.js Documentation](https://www.solidjs.com/)
- [Solid.js JSX Guide](https://www.solidjs.com/docs/latest/api#jsx)
- [Leptos Documentation](https://leptos.dev/)
- 既存のLeptos版コンポーネント: `src/components/ui/`

