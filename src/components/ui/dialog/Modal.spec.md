# Modal Component Specification (Solid.js)

## Related Files

- Implementation: `src/components/ui/dialog/Modal.tsx`
- ModalHeader: `src/components/ui/dialog/ModalHeader.tsx`
- ModalBody: `src/components/ui/dialog/ModalBody.tsx`
- ModalFooter: `src/components/ui/dialog/ModalFooter.tsx`
- Types: `src/types/ui.ts`
- Original (Leptos): `src/components/ui/dialog/modal.rs`

## Related Documentation

- Issue: https://github.com/otomatty/development-tools/issues/136
- Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
- Original Issue: https://github.com/otomatty/development-tools/issues/114

## Requirements

### 責務

Modal コンポーネントモジュールは以下の責務を担当する：

1. **Modal** - モーダルダイアログコンテナ

   - Portal レンダリング（body 要素にレンダリング）
   - ESC キーで閉じる機能
   - オーバーレイクリックで閉じる機能
   - 6 つのサイズ（Small, Medium, Large, XLarge, 2XL, Full）をサポート
   - カスタマイズ可能なボーダークラス
   - アニメーション対応（fade-in/scale-in）

2. **ModalHeader** - モーダルヘッダー
   - 一貫したヘッダーレイアウト
   - オプショナルな閉じるボタン

3. **ModalBody** - モーダルボディ
   - スクロール可能なコンテンツエリア
   - カスタマイズ可能なクラス

4. **ModalFooter** - モーダルフッター
   - アクションボタン用のフッターエリア

### 状態構造

#### ModalSize

| Size   | Max Width | 用途           |
| ------ | --------- | -------------- |
| sm     | max-w-sm  | 小さなモーダル |
| md     | max-w-md  | 中サイズ（デフォルト） |
| lg     | max-w-lg  | 大きなモーダル |
| xl     | max-w-xl  | 特大モーダル   |
| 2xl    | max-w-2xl | 2XL モーダル   |
| full   | max-w-4xl | フル幅モーダル |

### 公開 API

```typescript
export { Modal, ModalHeader, ModalBody, ModalFooter } from './Modal';
export type {
  ModalProps,
  ModalHeaderProps,
  ModalBodyProps,
  ModalFooterProps,
  ModalSize,
} from '../../../types/ui';
```

### スタイリング仕様

- オーバーレイ: `fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm`
- モーダルコンテナ: `bg-dt-card rounded-2xl w-full mx-4 shadow-xl`
- アニメーション: `animate-fade-in`（オーバーレイ）、`animate-scale-in`（モーダル本体）

## Test Cases

### TC-001: Modal 基本表示

- **Given**: Modal が visible=true で初期化
- **When**: レンダリングされる
- **Then**: Portal で body 要素にレンダリングされ、モーダルが表示される

### TC-002: Modal ESC キーで閉じる

- **Given**: Modal が visible=true、closeOnEscape=true で初期化
- **When**: ESC キーが押される
- **Then**: onClose コールバックが呼び出される

### TC-003: Modal オーバーレイクリックで閉じる

- **Given**: Modal が visible=true、closeOnOverlay=true で初期化
- **When**: オーバーレイがクリックされる
- **Then**: onClose コールバックが呼び出される

### TC-004: Modal コンテンツクリックで閉じない

- **Given**: Modal が visible=true、closeOnOverlay=true で初期化
- **When**: モーダルコンテンツ（オーバーレイ以外）がクリックされる
- **Then**: onClose コールバックは呼び出されない

### TC-005: ModalSize スタイル適用

- **Given**: 各サイズの Modal が初期化
- **When**: レンダリングされる
- **Then**: 対応する max-width クラスが適用される

### TC-006: ModalHeader 閉じるボタン表示

- **Given**: ModalHeader が onClose コールバック付きで初期化
- **When**: レンダリングされる
- **Then**: 閉じるボタン（✕）が表示される

### TC-007: ModalHeader 閉じるボタンなし

- **Given**: ModalHeader が onClose なしで初期化
- **When**: レンダリングされる
- **Then**: 閉じるボタンが表示されない

### TC-008: ModalBody スクロール可能

- **Given**: ModalBody に長いコンテンツが含まれる
- **When**: レンダリングされる
- **Then**: overflow-y-auto クラスが適用され、スクロール可能になる

### TC-009: ModalFooter レイアウト

- **Given**: ModalFooter に複数のボタンが含まれる
- **When**: レンダリングされる
- **Then**: ボタンが右寄せで、gap-3 で配置される

