# 実装計画: Phase3-2 優先度中のUIコンポーネントの移行

**作成日**: 2025-12-06  
**関連 Issue**: [#136](https://github.com/otomatty/development-tools/issues/136)  
**親 Issue**: [#129](https://github.com/otomatty/development-tools/issues/129)  
**依存 Issue**: Phase 3-1（基本UIコンポーネントの移行）  
**ステータス**: 計画中

---

## 1. 概要

Phase 3-1で実装した基本UIコンポーネントに続き、優先度中のコンポーネント（Card, Badge, Spinner）をLeptos（Rust）からSolid.js（TypeScript）に移行する。

### 移行の目的

- **基本UIコンポーネントの拡充**: Phase 3-1で実装したコンポーネントを補完する
- **一貫性の確保**: 既存のSolid.jsコンポーネントと同じパターンで実装
- **型安全性の向上**: TypeScriptによる型チェックでバグを早期発見

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

### 2.1 優先度中のコンポーネント（Phase 3-2で実装）

| コンポーネント | 現在のパス | 新規パス | 説明 |
| -------------- | ---------- | -------- | ---- |
| **Card** | `src/components/ui/card/card.rs` | `src/components/ui/card/Card.tsx` | カードコンテナ |
| **CardHeader** | `src/components/ui/card/card.rs` | `src/components/ui/card/Card.tsx` | カードヘッダー |
| **CardBody** | `src/components/ui/card/card.rs` | `src/components/ui/card/Card.tsx` | カードボディ |
| **CardFooter** | `src/components/ui/card/card.rs` | `src/components/ui/card/Card.tsx` | カードフッター |
| **Badge** | `src/components/ui/badge/badge.rs` | `src/components/ui/badge/Badge.tsx` | バッジ表示 |
| **Spinner** | `src/components/ui/feedback/loading.rs` | `src/components/ui/feedback/Spinner.tsx` | ローディングスピナー |

---

## 3. 実装フェーズ

### Phase 1: ディレクトリ構造と型定義（0.5日）

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P1-01 | `src/types/ui.ts` | Card, Badge, Spinnerの型定義追加 | 未着手 |
| P1-02 | `src/components/ui/card/index.ts` | Card関連のエクスポート | 未着手 |
| P1-03 | `src/components/ui/badge/index.ts` | Badgeのエクスポート | 未着手 |
| P1-04 | `src/components/ui/feedback/index.ts` | Spinnerのエクスポート追加 | 未着手 |

### Phase 2: Cardコンポーネント（1日）

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P2-01 | `src/components/ui/card/Card.spec.md` | 仕様書作成 | 未着手 |
| P2-02 | `src/components/ui/card/Card.tsx` | Cardコンポーネント実装 | 未着手 |
| P2-03 | `src/components/ui/card/Card.tsx` | CardHeaderコンポーネント実装 | 未着手 |
| P2-04 | `src/components/ui/card/Card.tsx` | CardBodyコンポーネント実装 | 未着手 |
| P2-05 | `src/components/ui/card/Card.tsx` | CardFooterコンポーネント実装 | 未着手 |
| P2-06 | `src/components/ui/card/index.ts` | エクスポート設定 | 未着手 |

### Phase 3: Badgeコンポーネント（0.5日）

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P3-01 | `src/components/ui/badge/Badge.spec.md` | 仕様書作成 | 未着手 |
| P3-02 | `src/components/ui/badge/Badge.tsx` | Badgeコンポーネント実装 | 未着手 |
| P3-03 | `src/components/ui/badge/index.ts` | エクスポート設定 | 未着手 |

### Phase 4: Spinnerコンポーネント（0.5日）

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P4-01 | `src/components/ui/feedback/Spinner.spec.md` | 仕様書作成 | 未着手 |
| P4-02 | `src/components/ui/feedback/Spinner.tsx` | Spinnerコンポーネント実装 | 未着手 |
| P4-03 | `src/components/ui/feedback/index.ts` | エクスポート設定更新 | 未着手 |

### Phase 5: 統合テスト・ドキュメント更新（0.5日）

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P5-01 | 統合テスト | 全コンポーネントの統合動作確認 | 未着手 |
| P5-02 | `docs/ARCHITECTURE.md` | アーキテクチャドキュメント更新 | 未着手 |
| P5-03 | `docs/05_logs/2025_12/YYYYMMDD/phase3-2-medium-priority-components.md` | 実装ログ作成 | 未着手 |

---

## 4. 技術的な実装詳細

### 4.1 Cardコンポーネント

#### 想定される機能

- **Card**: カードコンテナ（背景、ボーダー、角丸、影）
- **CardHeader**: カードヘッダー（タイトル、アクション）
- **CardBody**: カードボディ（コンテンツエリア）
- **CardFooter**: カードフッター（アクションボタン等）

#### 想定される型定義

```typescript
export interface CardProps {
  children: JSX.Element;
  class?: string;
  variant?: 'default' | 'outlined' | 'elevated';
}

export interface CardHeaderProps {
  children: JSX.Element;
  class?: string;
}

export interface CardBodyProps {
  children: JSX.Element;
  class?: string;
}

export interface CardFooterProps {
  children: JSX.Element;
  class?: string;
}
```

### 4.2 Badgeコンポーネント

#### 想定される機能

- バリアント（success, error, warning, info等）
- サイズ（sm, md, lg）
- アイコン対応

#### 想定される型定義

```typescript
export type BadgeVariant = 'success' | 'error' | 'warning' | 'info' | 'default';
export type BadgeSize = 'sm' | 'md' | 'lg';

export interface BadgeProps {
  children: JSX.Element | string;
  variant?: BadgeVariant;
  size?: BadgeSize;
  class?: string;
}
```

### 4.3 Spinnerコンポーネント

#### 想定される機能

- サイズ（sm, md, lg）
- カスタマイズ可能な色
- テキスト表示オプション

#### 想定される型定義

```typescript
export type SpinnerSize = 'sm' | 'md' | 'lg';

export interface SpinnerProps {
  size?: SpinnerSize;
  color?: string;
  text?: string;
  class?: string;
}
```

---

## 5. 工数見積もり

| フェーズ | 内容 | 見積もり | ステータス |
| -------- | ---- | -------- | ---------- |
| Phase 1 | ディレクトリ構造と型定義 | 0.5日 | 未着手 |
| Phase 2 | Cardコンポーネント | 1日 | 未着手 |
| Phase 3 | Badgeコンポーネント | 0.5日 | 未着手 |
| Phase 4 | Spinnerコンポーネント | 0.5日 | 未着手 |
| Phase 5 | 統合テスト・ドキュメント更新 | 0.5日 | 未着手 |
| **合計** | | **3日** | **未着手** |

---

## 6. 完了条件

- [ ] Card / CardHeader / CardBody / CardFooterがSolid.jsで実装されている
- [ ] BadgeがSolid.jsで実装されている
- [ ] SpinnerがSolid.jsで実装されている
- [ ] 各コンポーネントに.spec.mdが存在する
- [ ] TypeScriptの型が正しく定義されている
- [ ] コンポーネントが独立してレンダリングできる
- [ ] 実装計画の進捗状況を更新
- [ ] ARCHITECTURE.mdを更新
- [ ] 実装ログを作成

---

## 7. 次のステップ

Phase 3-2完了後：

1. **Phase 3-3**: 優先度低のコンポーネント（AnimatedEmoji, ConfirmDialog）を移行
2. **Phase 4**: 機能コンポーネント（features/）の移行
3. **Phase 5**: Leptos版の完全削除

---

## 8. 参考資料

- [Solid.js Documentation](https://www.solidjs.com/)
- [Solid.js JSX Guide](https://www.solidjs.com/docs/latest/api#jsx)
- Phase 3-1実装計画: `docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md`
- Phase 3-1実装ログ: `docs/05_logs/2025_12/20251206/phase3-1-ui-components-migration.md`
- Issue: https://github.com/otomatty/development-tools/issues/136

