# OfflineBanner Component Specification

## Related Files

- Implementation: `src/components/layouts/OfflineBanner/OfflineBanner.tsx`
- Network Store: `src/stores/networkStore.ts`
- Types: `src/types/network.ts`
- Original (Leptos): `src/components/network_status.rs`, `src/components/ui/feedback/offline_banner.rs`

## Related Documentation

- Issue: https://github.com/otomatty/development-tools/issues/137
- Network Status Spec: `src/components/network_status.spec.md`

## Requirements

### 責務

OfflineBanner コンポーネントは以下の責務を担当する：

1. **オフライン状態の表示** - ネットワークがオフラインのときに警告バナーを表示
2. **最終オンライン時刻の表示** - 最後にオンラインだった時刻を表示（利用可能な場合）
3. **視覚的な警告** - ユーザーにオフライン状態を明確に伝える

### 状態構造

#### NetworkState

```typescript
interface NetworkState {
  isOnline: boolean;
  lastCheckedAt: string | null;
  lastOnlineAt: string | null;
}
```

### 公開 API

```typescript
export { OfflineBanner } from './OfflineBanner';
```

### スタイリング仕様

- **バナー全体**: `bg-amber-500/90 text-amber-950 px-4 py-2 text-sm flex items-center justify-center gap-2`
- **アイコン**: `w-4 h-4` (alert-triangle)
- **最終オンライン時刻**: `text-amber-800 text-xs`

### 動作仕様

- **表示条件**: `isOnline === false` のときのみ表示
- **非表示条件**: `isOnline === true` のときは非表示（レンダリングしない）
- **最終オンライン時刻**: `lastOnlineAt` が存在する場合のみ表示

## Test Cases

### TC-001: オフライン時の表示

- **Given**: ネットワーク状態が `isOnline: false` である
- **When**: OfflineBanner がレンダリングされる
- **Then**: 警告バナーが表示される

### TC-002: オンライン時の非表示

- **Given**: ネットワーク状態が `isOnline: true` である
- **When**: OfflineBanner がレンダリングされる
- **Then**: バナーが表示されない（Show コンポーネントで非表示）

### TC-003: 最終オンライン時刻の表示

- **Given**: ネットワーク状態が `isOnline: false` かつ `lastOnlineAt: "2025-11-30T12:34:56.789Z"` である
- **When**: OfflineBanner がレンダリングされる
- **Then**: 「最終オンライン: 12:34」が表示される

### TC-004: 最終オンライン時刻がない場合

- **Given**: ネットワーク状態が `isOnline: false` かつ `lastOnlineAt: null` である
- **When**: OfflineBanner がレンダリングされる
- **Then**: 最終オンライン時刻は表示されない

### TC-005: タイムスタンプフォーマット

- **Given**: `lastOnlineAt: "2025-11-30T12:34:56.789Z"` である
- **When**: formatTimestamp 関数が呼び出される
- **Then**: "12:34" が返される

### TC-006: ネットワーク状態の変更

- **Given**: ネットワーク状態が `isOnline: true` から `isOnline: false` に変更される
- **When**: OfflineBanner が再レンダリングされる
- **Then**: バナーが表示される

### TC-007: アイコンの表示

- **Given**: ネットワーク状態が `isOnline: false` である
- **When**: OfflineBanner がレンダリングされる
- **Then**: alert-triangle アイコンが表示される

