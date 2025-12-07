# MainLayout Component Specification

## Related Files

- Implementation: `src/components/layouts/MainLayout/MainLayout.tsx`
- Sidebar: `src/components/layouts/Sidebar/Sidebar.tsx`
- OfflineBanner: `src/components/layouts/OfflineBanner/OfflineBanner.tsx`
- App: `src/App.tsx`
- Original (Leptos): `src/app.rs`

## Related Documentation

- Issue: https://github.com/otomatty/development-tools/issues/137
- Sidebar Spec: `src/components/layouts/Sidebar/Sidebar.spec.md`
- OfflineBanner Spec: `src/components/layouts/OfflineBanner/OfflineBanner.spec.md`

## Requirements

### 責務

MainLayout コンポーネントは以下の責務を担当する：

1. **アプリ全体のレイアウト** - Sidebar とメインコンテンツエリアを配置
2. **オフラインバナーの配置** - メインコンテンツエリアの上部に OfflineBanner を配置
3. **子コンポーネントの表示** - ルーターで定義されたページコンポーネントを表示

### 状態構造

MainLayout は状態を持たない。`ParentComponent` を使用して children を受け取る。

### 公開 API

```typescript
export { MainLayout } from './MainLayout';
```

### スタイリング仕様

- **レイアウト全体**: `flex h-screen bg-dt-bg`
- **メインコンテンツエリア**: `flex-1 flex flex-col overflow-hidden`

### レイアウト構造

```
MainLayout
├── Sidebar (固定幅 256px)
└── main (flex-1)
    ├── OfflineBanner (条件付き表示)
    └── children (ページコンテンツ)
```

## Test Cases

### TC-001: MainLayout 基本レンダリング

- **Given**: MainLayout が children と共に初期化される
- **When**: レンダリングされる
- **Then**: Sidebar とメインコンテンツエリアが表示される

### TC-002: Children の表示

- **Given**: MainLayout が children と共に初期化される
- **When**: レンダリングされる
- **Then**: children がメインコンテンツエリアに表示される

### TC-003: OfflineBanner の配置

- **Given**: MainLayout がレンダリングされる
- **When**: オフライン状態である
- **Then**: OfflineBanner がメインコンテンツエリアの上部に表示される

### TC-004: Sidebar の表示

- **Given**: MainLayout がレンダリングされる
- **When**: レンダリングされる
- **Then**: Sidebar が左側に固定表示される

### TC-005: レスポンシブ対応（将来実装）

- **Given**: 画面幅が小さい場合
- **When**: MainLayout がレンダリングされる
- **Then**: モバイルレイアウトが適用される（将来実装）

### TC-006: App.tsx との統合

- **Given**: App.tsx で MainLayout を使用
- **When**: ルーターでページが遷移する
- **Then**: 各ページが MainLayout 内に正しく表示される

### TC-007: レイアウトの高さ

- **Given**: MainLayout がレンダリングされる
- **When**: レンダリングされる
- **Then**: レイアウト全体の高さが画面の高さ（h-screen）になる

