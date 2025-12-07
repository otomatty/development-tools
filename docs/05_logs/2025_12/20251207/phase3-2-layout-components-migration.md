# Phase3-2: レイアウトコンポーネントの移行 - 実装ログ

**作成日**: 2025-12-07  
**関連 Issue**: [#137](https://github.com/otomatty/development-tools/issues/137)  
**親 Issue**: [#129](https://github.com/otomatty/development-tools/issues/129)  
**ステータス**: 完了

---

## 概要

Sidebar、MainLayout、OfflineBannerなどのレイアウトコンポーネントをLeptos（Rust）からSolid.js（TypeScript）に移行しました。また、アイコンライブラリ（lucide-solid）を導入し、既存のSVGアイコンを置き換えました。

## 実装内容

### Phase 1: アイコンライブラリの導入とIconコンポーネントの実装

#### 1.1 依存関係の追加
- `package.json`に`lucide-solid`（v0.460.0）を追加

#### 1.2 Iconコンポーネントの実装
- `src/components/icons/Icon.tsx`を作成
- lucide-solidのアイコンをラップするコンポーネント
- 既存のアイコン名（"home", "settings", "kanban"など）をlucide-solidのアイコン名にマッピング
- 51個のアイコン名をマッピング（既存のicons.rsの全アイコンに対応）

#### 1.3 型定義の追加
- `src/types/ui.ts`に`IconProps`型を追加

**実装ファイル**:
- `package.json`
- `src/components/icons/Icon.tsx`
- `src/components/icons/index.ts`
- `src/types/ui.ts`

### Phase 2: Sidebarコンポーネントの実装

#### 2.1 SidebarItemコンポーネント
- `src/components/layouts/Sidebar/SidebarItem.tsx`を作成
- ナビゲーション項目を表示するコンポーネント
- アクティブ状態のハイライト機能
- `@solidjs/router`の`A`コンポーネントを使用
- `useLocation`でアクティブな項目を判定

#### 2.2 Sidebarコンポーネント
- `src/components/layouts/Sidebar/Sidebar.tsx`を作成
- ヘッダー（ロゴ、アプリ名）
- ナビゲーション項目（Home, Projects, Issues, Mock Server, Settings）
- フッター（バージョン、設定ボタン）
- `useLocation`でアクティブな項目を判定

#### 2.3 仕様書の作成
- `src/components/layouts/Sidebar/Sidebar.spec.md`を作成

**実装ファイル**:
- `src/components/layouts/Sidebar/Sidebar.tsx`
- `src/components/layouts/Sidebar/SidebarItem.tsx`
- `src/components/layouts/Sidebar/index.ts`
- `src/components/layouts/Sidebar/Sidebar.spec.md`
- `src/components/layouts/index.ts`

### Phase 3: OfflineBannerコンポーネントの実装

#### 3.1 OfflineBannerコンポーネント
- `src/components/layouts/OfflineBanner/OfflineBanner.tsx`を作成
- `useNetworkStatus`フックでネットワーク状態を取得
- オフライン時のみ表示
- 最終オンライン時刻を表示
- 警告アイコンとメッセージを表示

#### 3.2 仕様書の作成
- `src/components/layouts/OfflineBanner/OfflineBanner.spec.md`を作成

**実装ファイル**:
- `src/components/layouts/OfflineBanner/OfflineBanner.tsx`
- `src/components/layouts/OfflineBanner/index.ts`
- `src/components/layouts/OfflineBanner/OfflineBanner.spec.md`
- `src/components/layouts/index.ts`（更新）

### Phase 4: MainLayoutコンポーネントの実装

#### 4.1 MainLayoutコンポーネント
- `src/components/layouts/MainLayout/MainLayout.tsx`を作成
- Sidebarとメインコンテンツエリアを配置
- OfflineBannerをメインコンテンツエリアの上部に配置
- `ParentComponent`を使用してchildrenを表示

#### 4.2 App.tsxの更新
- `src/App.tsx`を更新してMainLayoutを使用
- Router内でMainLayoutでラップ
- 各ページをMainLayoutのchildrenとして表示

#### 4.3 仕様書の作成
- `src/components/layouts/MainLayout/MainLayout.spec.md`を作成

**実装ファイル**:
- `src/components/layouts/MainLayout/MainLayout.tsx`
- `src/components/layouts/MainLayout/index.ts`
- `src/components/layouts/MainLayout/MainLayout.spec.md`
- `src/App.tsx`（更新）
- `src/components/layouts/index.ts`（更新）

### Phase 5: 統合テスト・ドキュメント更新

#### 5.1 統合テスト
- 全コンポーネントの統合動作確認
- リンターエラーの確認（エラーなし）

#### 5.2 ドキュメント更新
- `docs/ARCHITECTURE.md`を更新
  - レイアウトコンポーネントのセクションを追加
  - ディレクトリ構造を更新
  - アイコンコンポーネントの情報を追加

**実装ファイル**:
- `docs/ARCHITECTURE.md`（更新）

## 技術的な実装詳細

### アイコン名のマッピング

既存のアイコン名をlucide-solidのアイコン名にマッピング：

```typescript
const iconNameMap: Record<string, keyof typeof LucideIcons> = {
  'home': 'Home',
  'settings': 'Settings',
  'kanban': 'LayoutGrid',
  'folder': 'Folder',
  'list': 'List',
  'server': 'Server',
  // ... 51個のマッピング
};
```

### Sidebarのナビゲーション項目

```typescript
const navItems: NavItem[] = [
  { path: '/', label: 'ホーム', icon: 'home', exact: true },
  { path: '/projects', label: 'プロジェクト', icon: 'folder' },
  { path: '/issues', label: 'Issue', icon: 'list' },
  { path: '/mock-server', label: 'Mock Server', icon: 'server' },
  { path: '/settings', label: '設定', icon: 'settings' },
];
```

### MainLayoutの構造

```typescript
export const MainLayout: ParentComponent = (props) => {
  return (
    <div class="flex h-screen bg-dt-bg">
      <Sidebar />
      <main class="flex-1 flex flex-col overflow-hidden">
        <OfflineBanner />
        {props.children}
      </main>
    </div>
  );
};
```

## 実装結果

### 完了したタスク

- [x] lucide-solidがインストールされている
- [x] IconコンポーネントがSolid.jsで実装されている
- [x] Sidebarが正しくナビゲーションとして機能する
- [x] MainLayoutで全ページが統一的にラップされる
- [x] アクティブなナビゲーション項目がハイライトされる
- [x] オフライン時にバナーが表示される
- [x] アイコンが正しく表示される
- [x] 各コンポーネントに.spec.mdが存在する
- [x] TypeScriptの型が正しく定義されている
- [x] ARCHITECTURE.mdを更新
- [x] 実装ログを作成

### 実装ファイル一覧

**新規作成**:
- `src/components/icons/Icon.tsx`
- `src/components/icons/index.ts`
- `src/components/layouts/Sidebar/Sidebar.tsx`
- `src/components/layouts/Sidebar/SidebarItem.tsx`
- `src/components/layouts/Sidebar/index.ts`
- `src/components/layouts/Sidebar/Sidebar.spec.md`
- `src/components/layouts/OfflineBanner/OfflineBanner.tsx`
- `src/components/layouts/OfflineBanner/index.ts`
- `src/components/layouts/OfflineBanner/OfflineBanner.spec.md`
- `src/components/layouts/MainLayout/MainLayout.tsx`
- `src/components/layouts/MainLayout/index.ts`
- `src/components/layouts/MainLayout/MainLayout.spec.md`
- `src/components/layouts/index.ts`

**更新**:
- `package.json`（lucide-solid追加）
- `src/types/ui.ts`（IconProps型追加）
- `src/App.tsx`（MainLayout統合）
- `docs/ARCHITECTURE.md`（レイアウトコンポーネント情報追加）

## 注意事項

- 既存のLeptos版のコンポーネント（`src/components/sidebar.rs`、`src/components/icons.rs`など）は、移行完了まで残しておく
- レスポンシブ対応は後のフェーズで実装予定（issue #137の注意事項に記載）
- アイコン名のマッピングは、既存のコードとの互換性を保つ

## 次のステップ

Phase 3-2完了後：

1. **Phase 3-3**: 優先度低のコンポーネント（AnimatedEmoji, ConfirmDialog）を移行
2. **Phase 4**: 機能コンポーネント（features/）の移行
3. **Phase 5**: Leptos版の完全削除

## 参考資料

- [Solid.js Documentation](https://www.solidjs.com/)
- [@solidjs/router Documentation](https://github.com/solidjs/solid-router)
- [lucide-solid Documentation](https://lucide.dev/guide/packages/lucide-solid)
- Phase 3-1実装計画: `docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md`
- Issue: https://github.com/otomatty/development-tools/issues/137

