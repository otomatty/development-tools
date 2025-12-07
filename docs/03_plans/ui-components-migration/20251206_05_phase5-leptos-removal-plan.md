# 実装計画: Phase 5 Leptos版コンポーネントの削除

**作成日**: 2025-12-06  
**関連 Issue**: [#129](https://github.com/otomatty/development-tools/issues/129)  
**依存 Issue**: Phase 3-1, 3-2, 3-3, Phase 4（全コンポーネントの移行完了）  
**ステータス**: 計画中

---

## 1. 概要

Phase 3-1, 3-2, 3-3, Phase 4でSolid.js版への移行が完了した後、Leptos版のコンポーネントを削除する。

### 削除の目的

- **コードベースの整理**: 不要になったLeptos版コンポーネントを削除
- **保守性の向上**: 重複コードを削除し、保守性を向上
- **ビルド時間の短縮**: 不要なRustコードを削除し、ビルド時間を短縮

### 基本原則

| 原則 | 説明 |
| ---- | ---- |
| **段階的削除** | コンポーネントごとに段階的に削除 |
| **使用箇所の確認** | 削除前に使用箇所を確認 |
| **テストの実行** | 削除後にテストを実行して動作確認 |
| **ドキュメント更新** | 削除後にドキュメントを更新 |

---

## 2. 削除対象コンポーネント

### 2.1 UIコンポーネント（Phase 3で移行済み）

| コンポーネント | Leptos版パス | 削除条件 | ステータス |
| -------------- | ------------ | -------- | ---------- |
| **Button** | `src/components/ui/button/button.rs` | Solid.js版が全使用箇所で使用されている | 未着手 |
| **Input** | `src/components/ui/form/input.rs` | Solid.js版が全使用箇所で使用されている | 未着手 |
| **Modal** | `src/components/ui/dialog/modal.rs` | Solid.js版が全使用箇所で使用されている | 未着手 |
| **DropdownMenu** | `src/components/ui/dropdown/dropdown_menu.rs` | Solid.js版が全使用箇所で使用されている | 未着手 |
| **Toast** | `src/components/ui/feedback/toast.rs` | Solid.js版が全使用箇所で使用されている | 未着手 |
| **Card** | `src/components/ui/card/card.rs` | Solid.js版が全使用箇所で使用されている | 未着手 |
| **Badge** | `src/components/ui/badge/badge.rs` | Solid.js版が全使用箇所で使用されている | 未着手 |
| **Spinner** | `src/components/ui/feedback/loading.rs` | Solid.js版が全使用箇所で使用されている | 未着手 |
| **AnimatedEmoji** | `src/components/animated_emoji.rs` | Solid.js版が全使用箇所で使用されている | 未着手 |
| **ConfirmDialog** | `src/components/confirm_dialog.rs` | Solid.js版が全使用箇所で使用されている | 未着手 |

### 2.2 機能コンポーネント（Phase 4で移行済み）

Phase 4で移行した機能コンポーネントのLeptos版も削除対象です。

---

## 3. 実装フェーズ

### Phase 1: 使用箇所の確認（1日）

| タスク | 内容 | ステータス |
| ------ | ---- | ---------- |
| P1-01 | 各Leptos版コンポーネントの使用箇所を検索 | 未着手 |
| P1-02 | 使用箇所のリストを作成 | 未着手 |
| P1-03 | 削除可能なコンポーネントを特定 | 未着手 |

### Phase 2: UIコンポーネントの削除（1日）

| タスク | 内容 | ステータス |
| ------ | ---- | ---------- |
| P2-01 | Button（Leptos版）の削除 | 未着手 |
| P2-02 | Input（Leptos版）の削除 | 未着手 |
| P2-03 | Modal（Leptos版）の削除 | 未着手 |
| P2-04 | DropdownMenu（Leptos版）の削除 | 未着手 |
| P2-05 | Toast（Leptos版）の削除 | 未着手 |
| P2-06 | Card（Leptos版）の削除 | 未着手 |
| P2-07 | Badge（Leptos版）の削除 | 未着手 |
| P2-08 | Spinner（Leptos版）の削除 | 未着手 |
| P2-09 | AnimatedEmoji（Leptos版）の削除 | 未着手 |
| P2-10 | ConfirmDialog（Leptos版）の削除 | 未着手 |

### Phase 3: 機能コンポーネントの削除（1-2日）

Phase 4で移行した機能コンポーネントのLeptos版を削除します。

### Phase 4: モジュール定義の更新（0.5日）

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P4-01 | `src/components/mod.rs` | 削除したコンポーネントのエクスポートを削除 | 未着手 |
| P4-02 | `src/components/ui/mod.rs` | UIコンポーネントのエクスポートを更新 | 未着手 |

### Phase 5: テスト・ドキュメント更新（0.5日）

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P5-01 | テスト | 全機能の動作確認 | 未着手 |
| P5-02 | `docs/ARCHITECTURE.md` | アーキテクチャドキュメント更新 | 未着手 |
| P5-03 | `docs/05_logs/2025_12/YYYYMMDD/phase5-leptos-removal.md` | 実装ログ作成 | 未着手 |

---

## 4. 削除手順

### 4.1 各コンポーネントの削除手順

1. **使用箇所の確認**: `grep`や`ripgrep`で使用箇所を検索
2. **Solid.js版への置き換え確認**: 全使用箇所がSolid.js版を使用していることを確認
3. **ファイルの削除**: Leptos版のファイルを削除
4. **モジュール定義の更新**: `mod.rs`から削除したコンポーネントのエクスポートを削除
5. **テストの実行**: 削除後にテストを実行して動作確認

### 4.2 注意事項

- **段階的削除**: 一度に全てを削除せず、コンポーネントごとに削除
- **使用箇所の確認**: 削除前に必ず使用箇所を確認
- **テストの実行**: 削除後に必ずテストを実行
- **ドキュメント更新**: 削除後にドキュメントを更新

---

## 5. 工数見積もり

| フェーズ | 内容 | 見積もり | ステータス |
| -------- | ---- | -------- | ---------- |
| Phase 1 | 使用箇所の確認 | 1日 | 未着手 |
| Phase 2 | UIコンポーネントの削除 | 1日 | 未着手 |
| Phase 3 | 機能コンポーネントの削除 | 1-2日 | 未着手 |
| Phase 4 | モジュール定義の更新 | 0.5日 | 未着手 |
| Phase 5 | テスト・ドキュメント更新 | 0.5日 | 未着手 |
| **合計** | | **4-5日** | **未着手** |

---

## 6. 完了条件

- [ ] 全Leptos版UIコンポーネントが削除されている
- [ ] 全Leptos版機能コンポーネントが削除されている
- [ ] モジュール定義が更新されている
- [ ] テストが全て通過している
- [ ] ドキュメントが更新されている
- [ ] 実装ログが作成されている

---

## 7. 参考資料

- Phase 3-1実装計画: `docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md`
- Phase 3-2実装計画: `docs/03_plans/ui-components-migration/20251206_02_phase3-2-medium-priority-components-plan.md`
- Phase 3-3実装計画: `docs/03_plans/ui-components-migration/20251206_03_phase3-3-low-priority-components-plan.md`
- Phase 4実装計画: `docs/03_plans/ui-components-migration/20251206_04_phase4-feature-components-plan.md`
- Issue: https://github.com/otomatty/development-tools/issues/129

