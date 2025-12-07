# 実装計画: Phase3-3 優先度低のUIコンポーネントの移行

**作成日**: 2025-12-06  
**関連 Issue**: [#136](https://github.com/otomatty/development-tools/issues/136)  
**親 Issue**: [#129](https://github.com/otomatty/development-tools/issues/129)  
**依存 Issue**: Phase 3-2（優先度中のUIコンポーネントの移行）  
**ステータス**: 計画中

---

## 1. 概要

Phase 3-2で実装した優先度中のコンポーネントに続き、優先度低のコンポーネント（AnimatedEmoji, ConfirmDialog）をLeptos（Rust）からSolid.js（TypeScript）に移行する。

### 移行の目的

- **UIコンポーネント移行の完了**: 基本UIコンポーネントの移行を完了させる
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

### 2.1 優先度低のコンポーネント（Phase 3-3で実装）

| コンポーネント | 現在のパス | 新規パス | 説明 |
| -------------- | ---------- | -------- | ---- |
| **AnimatedEmoji** | `src/components/animated_emoji.rs` | `src/components/AnimatedEmoji.tsx` | アニメーション絵文字 |
| **ConfirmDialog** | `src/components/confirm_dialog.rs` | `src/components/ConfirmDialog.tsx` | 確認ダイアログ |

---

## 3. 実装フェーズ

### Phase 1: ディレクトリ構造と型定義（0.5日）

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P1-01 | `src/types/ui.ts` | AnimatedEmoji, ConfirmDialogの型定義追加 | 未着手 |
| P1-02 | `src/components/AnimatedEmoji.tsx` | コンポーネントファイル作成準備 | 未着手 |
| P1-03 | `src/components/ConfirmDialog.tsx` | コンポーネントファイル作成準備 | 未着手 |

### Phase 2: AnimatedEmojiコンポーネント（1日）

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P2-01 | `src/components/AnimatedEmoji.spec.md` | 仕様書作成 | 未着手 |
| P2-02 | `src/components/AnimatedEmoji.tsx` | AnimatedEmojiコンポーネント実装 | 未着手 |
| P2-03 | テスト | 既存のLeptos版と同等の動作確認 | 未着手 |

### Phase 3: ConfirmDialogコンポーネント（1日）

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P3-01 | `src/components/ConfirmDialog.spec.md` | 仕様書作成 | 未着手 |
| P3-02 | `src/components/ConfirmDialog.tsx` | ConfirmDialogコンポーネント実装 | 未着手 |
| P3-03 | テスト | 既存のLeptos版と同等の動作確認 | 未着手 |

### Phase 4: 統合テスト・ドキュメント更新（0.5日）

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P4-01 | 統合テスト | 全コンポーネントの統合動作確認 | 未着手 |
| P4-02 | `docs/ARCHITECTURE.md` | アーキテクチャドキュメント更新 | 未着手 |
| P4-03 | `docs/05_logs/2025_12/YYYYMMDD/phase3-3-low-priority-components.md` | 実装ログ作成 | 未着手 |

---

## 4. 技術的な実装詳細

### 4.1 AnimatedEmojiコンポーネント

#### 想定される機能

- 絵文字のアニメーション表示
- アニメーションタイプ（bounce, spin, pulse等）
- サイズ調整

#### 想定される型定義

```typescript
export type EmojiAnimationType = 'bounce' | 'spin' | 'pulse' | 'none';

export interface AnimatedEmojiProps {
  emoji: string;
  animation?: EmojiAnimationType;
  size?: 'sm' | 'md' | 'lg';
  class?: string;
}
```

### 4.2 ConfirmDialogコンポーネント

#### 想定される機能

- Modalベースの確認ダイアログ
- タイトル、メッセージ、確認/キャンセルボタン
- カスタマイズ可能なボタンテキスト

#### 想定される型定義

```typescript
export interface ConfirmDialogProps {
  visible: boolean | Accessor<boolean>;
  onClose: () => void;
  onConfirm: () => void;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  confirmVariant?: 'primary' | 'danger';
  size?: ModalSize;
}
```

---

## 5. 工数見積もり

| フェーズ | 内容 | 見積もり | ステータス |
| -------- | ---- | -------- | ---------- |
| Phase 1 | ディレクトリ構造と型定義 | 0.5日 | 未着手 |
| Phase 2 | AnimatedEmojiコンポーネント | 1日 | 未着手 |
| Phase 3 | ConfirmDialogコンポーネント | 1日 | 未着手 |
| Phase 4 | 統合テスト・ドキュメント更新 | 0.5日 | 未着手 |
| **合計** | | **3日** | **未着手** |

---

## 6. 完了条件

- [ ] AnimatedEmojiがSolid.jsで実装されている
- [ ] ConfirmDialogがSolid.jsで実装されている
- [ ] 各コンポーネントに.spec.mdが存在する
- [ ] TypeScriptの型が正しく定義されている
- [ ] コンポーネントが独立してレンダリングできる
- [ ] 実装計画の進捗状況を更新
- [ ] ARCHITECTURE.mdを更新
- [ ] 実装ログを作成

---

## 7. 次のステップ

Phase 3-3完了後：

1. **Phase 4**: 機能コンポーネント（features/）の移行
2. **Phase 5**: Leptos版の完全削除

---

## 8. 参考資料

- [Solid.js Documentation](https://www.solidjs.com/)
- [Solid.js JSX Guide](https://www.solidjs.com/docs/latest/api#jsx)
- Phase 3-1実装計画: `docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md`
- Phase 3-2実装計画: `docs/03_plans/ui-components-migration/20251206_02_phase3-2-medium-priority-components-plan.md`
- Issue: https://github.com/otomatty/development-tools/issues/136

