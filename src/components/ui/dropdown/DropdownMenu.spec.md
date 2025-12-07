# DropdownMenu Component Specification (Solid.js)

## Related Files

- Implementation: `src/components/ui/dropdown/DropdownMenu.tsx`
- Types: `src/types/ui.ts`
- Original (Leptos): `src/components/ui/dropdown/dropdown_menu.rs`

## Related Documentation

- Issue: https://github.com/otomatty/development-tools/issues/136
- Plan: docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md
- Original Spec: `src/components/dropdown_menu.spec.md`
- Original Issue: https://github.com/otomatty/development-tools/issues/39

## Requirements

### 責務

DropdownMenu コンポーネントモジュールは以下の責務を担当する：

1. **DropdownMenu** - ドロップダウンメニューコンテナ

   - トリガー要素のクリックでメニューを開閉
   - メニュー外クリックで閉じる
   - ESC キーで閉じる
   - アニメーション対応（fade-in/slide-down）
   - 左/右配置のサポート

2. **DropdownMenuItem** - メニューアイテム
   - クリックハンドラー対応
   - danger プロパティ（赤色表示）
   - クリック後にメニューを自動的に閉じる

3. **DropdownMenuDivider** - メニュー区切り線
   - メニューアイテム間の区切り

### 状態構造

- `isOpen`: メニューの開閉状態（内部状態）

### 公開 API

```typescript
export { DropdownMenu, DropdownMenuItem, DropdownMenuDivider } from './DropdownMenu';
export type {
  DropdownMenuProps,
  DropdownMenuItemProps,
  DropdownMenuDividerProps,
} from '../../../types/ui';
```

### スタイリング仕様

#### トリガーボタン

- `p-2 text-dt-text-sub hover:text-gm-accent-cyan transition-colors rounded-lg`

#### メニューコンテナ

- `absolute right-0 top-full mt-2 min-w-[160px] bg-gm-bg-card/95 backdrop-blur-sm border border-gm-accent-cyan/20 rounded-lg shadow-lg z-50`

#### メニューアイテム（通常）

- `flex items-center gap-3 px-4 py-2 text-sm text-dt-text-main hover:bg-gm-accent-cyan/10 transition-colors cursor-pointer w-full text-left`

#### メニューアイテム（danger）

- `text-gm-error hover:bg-gm-error/10`（上記に追加）

## Test Cases

### TC-001: メニュー初期状態

- **Given**: DropdownMenu コンポーネントがマウントされた状態
- **When**: 初期表示時
- **Then**: メニューは閉じている（isOpen = false）

### TC-002: トリガークリックでメニュー開く

- **Given**: メニューが閉じている状態
- **When**: トリガーボタンをクリック
- **Then**: メニューが開く（isOpen = true）

### TC-003: トリガークリックでメニュー閉じる

- **Given**: メニューが開いている状態
- **When**: トリガーボタンをクリック
- **Then**: メニューが閉じる（isOpen = false）

### TC-004: メニュー外クリックで閉じる

- **Given**: メニューが開いている状態
- **When**: メニュー外の領域をクリック
- **Then**: メニューが閉じる

### TC-005: ESC キーでメニュー閉じる

- **Given**: メニューが開いている状態
- **When**: ESC キーを押下
- **Then**: メニューが閉じる

### TC-006: メニューアイテムクリック

- **Given**: メニューが開いている状態
- **When**: メニューアイテムをクリック
- **Then**: onClick ハンドラーが呼ばれる、メニューが閉じる

### TC-007: アニメーション有効時のトランジション

- **Given**: アニメーションが有効
- **When**: メニューを開く
- **Then**: フェードイン + スライドダウンアニメーションが適用される

### TC-008: アニメーション無効時の即時表示

- **Given**: アニメーションが無効
- **When**: メニューを開く
- **Then**: アニメーションなしで即座に表示される

### TC-009: danger プロパティの適用

- **Given**: DropdownMenuItem に danger=true を設定
- **When**: メニューアイテムが表示される
- **Then**: 赤色のスタイル（text-gm-error）が適用される

### TC-010: aria-expanded 属性の更新

- **Given**: メニューの開閉状態が変化
- **When**: isOpen が true/false に変更
- **Then**: トリガーボタンの aria-expanded 属性が対応する値に更新される

### TC-011: メニュー配置（left/right）

- **Given**: align="left" が指定
- **When**: メニューが表示される
- **Then**: left-0 クラスが適用される

