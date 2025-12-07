# Sidebar Component Specification

## Related Files

- Implementation: `src/components/layouts/Sidebar/Sidebar.tsx`
- SidebarItem: `src/components/layouts/Sidebar/SidebarItem.tsx`
- Types: `src/types/ui.ts`
- Original (Leptos): `src/components/sidebar.rs`

## Related Documentation

- Issue: https://github.com/otomatty/development-tools/issues/137
- Navigation Store: `src/stores/navigationStore.ts`
- Navigation Utility: `src/lib/navigation.ts`

## Requirements

### 責務

Sidebar コンポーネントは以下の責務を担当する：

1. **メインナビゲーション** - アプリケーション全体のナビゲーションを提供
2. **アクティブ状態の表示** - 現在のページに対応するナビゲーション項目をハイライト
3. **アプリヘッダー** - ロゴとアプリ名を表示
4. **フッター** - バージョン情報と設定ボタンを表示

### 状態構造

#### NavItem

```typescript
interface NavItem {
  path: string;      // ルートパス
  label: string;    // 表示ラベル
  icon: string;      // アイコン名
  exact?: boolean;   // 完全一致フラグ（デフォルト: false）
}
```

#### ナビゲーション項目

| Path | Label | Icon | Exact |
| ---- | ----- | ---- | ----- |
| `/` | ホーム | `home` | `true` |
| `/projects` | プロジェクト | `folder` | `false` |
| `/issues` | Issue | `list` | `false` |
| `/mock-server` | Mock Server | `server` | `false` |
| `/settings` | 設定 | `settings` | `false` |

### 公開 API

```typescript
export { Sidebar } from './Sidebar';
export { SidebarItem, type SidebarItemProps } from './SidebarItem';
```

### スタイリング仕様

- **サイドバー全体**: `w-64 bg-slate-900 border-r border-slate-700/50 flex flex-col h-full`
- **ヘッダー**: `p-4 border-b border-slate-700/50`
- **ロゴ**: `p-2 bg-gradient-to-br from-gm-accent-cyan to-gm-accent-purple rounded-lg`
- **アクティブ項目**: `bg-gradient-to-r from-gm-accent-cyan/20 to-gm-accent-purple/20 text-gm-accent-cyan border-l-2 border-gm-accent-cyan`
- **非アクティブ項目**: `text-slate-400 hover:bg-slate-800 hover:text-dt-text`
- **フッター**: `p-3 border-t border-slate-700/50`

## Test Cases

### TC-001: Sidebar 基本レンダリング

- **Given**: Sidebar が初期化される
- **When**: レンダリングされる
- **Then**: ヘッダー、ナビゲーション項目、フッターが表示される

### TC-002: アクティブ項目のハイライト

- **Given**: 現在のパスが `/projects` である
- **When**: Sidebar がレンダリングされる
- **Then**: 「プロジェクト」項目がアクティブスタイルで表示される

### TC-003: ルートパスの完全一致

- **Given**: 現在のパスが `/` である
- **When**: Sidebar がレンダリングされる
- **Then**: 「ホーム」項目のみがアクティブスタイルで表示される（`/projects` などは非アクティブ）

### TC-004: サブパスの部分一致

- **Given**: 現在のパスが `/projects/123` である
- **When**: Sidebar がレンダリングされる
- **Then**: 「プロジェクト」項目がアクティブスタイルで表示される（`exact=false` のため）

### TC-005: ナビゲーションリンクの動作

- **Given**: Sidebar がレンダリングされる
- **When**: ナビゲーション項目がクリックされる
- **Then**: 対応するページに遷移する

### TC-006: 設定ボタンのアクティブ状態

- **Given**: 現在のパスが `/settings` である
- **When**: Sidebar がレンダリングされる
- **Then**: フッターの設定ボタンがアクティブスタイルで表示される

### TC-007: アイコンの表示

- **Given**: Sidebar がレンダリングされる
- **When**: 各ナビゲーション項目が表示される
- **Then**: 対応するアイコンが正しく表示される

