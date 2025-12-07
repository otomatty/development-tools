# Toast Component Specification (Solid.js)

## Related Files

- Implementation: `src/components/ui/feedback/Toast.tsx`
- Types: `src/types/ui.ts`
- Hook: `src/hooks/useToast.ts` (to be created/updated)
- Original (Leptos): `src/components/ui/feedback/toast.rs`

## Related Documentation

- Issue: https://github.com/otomatty/development-tools/issues/136
- Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
- Original Spec: `src/components/ui/feedback/feedback.spec.md`
- Original Issue: https://github.com/otomatty/development-tools/issues/115

## Requirements

### 責務

Toast コンポーネントモジュールは以下の責務を担当する：

1. **Toast** - 一時的な通知表示

   - 4 つのタイプ（Success, Error, Info, Warning）をサポート
   - 自動非表示（時間経過、デフォルト3秒）
   - 固定位置表示（画面右下）
   - アクセシビリティ対応（role="alert", aria-live="polite"）
   - アニメーション対応（slideInUp）

2. **InlineToast** - インライン通知表示
   - パネル内での通知表示用
   - 同じ 4 タイプをサポート
   - 相対位置での表示

### 状態構造

#### ToastType

| Type    | アイコン | 背景色              | テキスト色          | 用途           |
| ------- | -------- | ------------------- | ------------------- | -------------- |
| success | ✓        | bg-green-900/90     | text-green-200      | 成功メッセージ |
| error   | ✗        | bg-red-900/90       | text-red-200        | エラーメッセージ |
| info    | ℹ        | bg-gm-accent-cyan/20 | text-gm-accent-cyan | 情報メッセージ |
| warning | ⚠        | bg-amber-900/90     | text-amber-200      | 警告メッセージ |

### 公開 API

```typescript
export { Toast, InlineToast } from './Toast';
export type { ToastType, ToastProps } from '../../../types/ui';
```

### スタイリング仕様

#### Toast（固定位置）

- 位置: `fixed bottom-6 right-6 z-50`
- 基本クラス: `flex items-center gap-3 px-5 py-3 rounded-xl backdrop-blur-sm`
- アニメーション: `animate-slideInUp`

#### InlineToast（相対位置）

- 基本クラス: `flex items-center gap-2 px-4 py-2.5 rounded-lg`
- アニメーション: `animate-fadeIn`

## Test Cases

### TC-001: Toast 表示/非表示

- **Given**: Toast が visible=true で初期化
- **When**: コンポーネントがレンダリング
- **Then**: Toast が画面に表示される

### TC-002: Toast 非表示状態

- **Given**: Toast が visible=false で初期化
- **When**: コンポーネントがレンダリング
- **Then**: Toast が画面に表示されない

### TC-003: Toast Success タイプスタイル

- **Given**: ToastType="success" が指定
- **When**: Toast がレンダリング
- **Then**: 緑色のスタイル（bg-green-900/90）が適用される

### TC-004: Toast Error タイプスタイル

- **Given**: ToastType="error" が指定
- **When**: Toast がレンダリング
- **Then**: 赤色のスタイル（bg-red-900/90）が適用される

### TC-005: Toast Info タイプスタイル

- **Given**: ToastType="info" が指定
- **When**: Toast がレンダリング
- **Then**: シアン色のスタイル（bg-gm-accent-cyan/20）が適用される

### TC-006: Toast Warning タイプスタイル

- **Given**: ToastType="warning" が指定
- **When**: Toast がレンダリング
- **Then**: アンバー色のスタイル（bg-amber-900/90）が適用される

### TC-007: Toast メッセージ表示

- **Given**: message="保存しました" が指定
- **When**: Toast がレンダリング
- **Then**: "保存しました" テキストが表示される

### TC-008: Toast アイコン表示

- **Given**: ToastType="success" が指定
- **When**: Toast がレンダリング
- **Then**: "✓" アイコンが表示される

### TC-009: InlineToast レンダリング

- **Given**: InlineToast が visible=true で初期化
- **When**: レンダリングされる
- **Then**: InlineToast が表示される

### TC-010: Toast 自動非表示

- **Given**: Toast が duration=3000 で初期化
- **When**: 3秒経過
- **Then**: onClose コールバックが呼び出される

### TC-011: Toast アクセシビリティ

- **Given**: Toast がレンダリングされる
- **When**: 表示される
- **Then**: role="alert" と aria-live="polite" が設定される

