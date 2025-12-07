# 実装計画: Phase 4 機能コンポーネントの移行

**作成日**: 2025-12-06  
**関連 Issue**: [#129](https://github.com/otomatty/development-tools/issues/129)  
**依存 Issue**: Phase 3-1, 3-2, 3-3（UIコンポーネントの移行）  
**ステータス**: 計画中

---

## 1. 概要

Phase 3で実装したUIコンポーネントを基盤として、機能コンポーネント（features/）をLeptos（Rust）からSolid.js（TypeScript）に移行する。

### 移行の目的

- **機能コンポーネントの移行**: ビジネスロジックを含む機能コンポーネントをSolid.jsに移行
- **UIコンポーネントの活用**: Phase 3で実装したSolid.js版UIコンポーネントを活用
- **段階的移行**: 機能ごとに段階的に移行し、リスクを最小化

### 基本原則

| 原則 | 説明 |
| ---- | ---- |
| **機能の完全再現** | Leptos版の全機能をSolid.js版で実装 |
| **UIコンポーネントの活用** | Phase 3で実装したSolid.js版UIコンポーネントを使用 |
| **型安全性** | TypeScriptで厳密な型定義 |
| **仕様書駆動** | 各コンポーネントに.spec.mdを作成 |
| **段階的移行** | 機能ごとに移行し、既存コードへの影響を最小化 |

---

## 2. 移行対象コンポーネント

### 2.1 機能コンポーネント一覧（調査が必要）

以下のコンポーネントが移行対象として考えられますが、詳細な調査が必要です：

| コンポーネント | 現在のパス | 新規パス | 説明 | 優先度 |
| -------------- | ---------- | -------- | ---- | ------ |
| **ProfileCard** | `src/components/home/profile_card.rs` | `src/pages/Home/ProfileCard.tsx` | プロフィールカード | 高 |
| **StatsDisplay** | `src/components/home/stats_display.rs` | `src/pages/Home/StatsDisplay.tsx` | 統計表示 | 高 |
| **ChallengeCard** | `src/components/home/challenge_card.rs` | `src/pages/Home/ChallengeCard.tsx` | チャレンジカード | 中 |
| **BadgeGrid** | `src/components/home/badge_grid.rs` | `src/pages/Home/BadgeGrid.tsx` | バッジ一覧 | 中 |
| **ContributionGraph** | `src/components/home/contribution_graph.rs` | `src/pages/Home/ContributionGraph.tsx` | コントリビューショングラフ | 中 |
| **XpNotification** | `src/components/home/xp_notification.rs` | `src/pages/Home/XpNotification.tsx` | XP通知 | 低 |
| **LoginCard** | `src/components/home/login_card.rs` | `src/pages/Home/LoginCard.tsx` | ログインUI | 低 |

**注意**: 上記は推測に基づくリストです。実際の移行前に詳細な調査が必要です。

---

## 3. 実装フェーズ（概要）

### Phase 1: 調査・分析（1日）

| タスク | 内容 | ステータス |
| ------ | ---- | ---------- |
| P1-01 | 移行対象コンポーネントの洗い出し | 未着手 |
| P1-02 | 各コンポーネントの依存関係の分析 | 未着手 |
| P1-03 | 移行優先順位の決定 | 未着手 |
| P1-04 | 実装計画の詳細化 | 未着手 |

### Phase 2: 優先度高のコンポーネント移行（3-5日）

優先度の高いコンポーネントから順に移行します。

### Phase 3: 優先度中のコンポーネント移行（2-3日）

優先度中のコンポーネントを移行します。

### Phase 4: 優先度低のコンポーネント移行（1-2日）

優先度低のコンポーネントを移行します。

### Phase 5: 統合テスト・ドキュメント更新（1日）

| タスク | ファイル | 内容 | ステータス |
| ------ | -------- | ---- | ---------- |
| P5-01 | 統合テスト | 全コンポーネントの統合動作確認 | 未着手 |
| P5-02 | `docs/ARCHITECTURE.md` | アーキテクチャドキュメント更新 | 未着手 |
| P5-03 | `docs/05_logs/2025_12/YYYYMMDD/phase4-feature-components.md` | 実装ログ作成 | 未着手 |

---

## 4. 技術的な考慮事項

### 4.1 状態管理

機能コンポーネントは状態管理が必要な場合があります：

- **Solid.jsストア**: `createStore`を使用したグローバル状態管理
- **Context API**: コンポーネント間での状態共有
- **Props**: 親コンポーネントからの状態受け渡し

### 4.2 API呼び出し

機能コンポーネントはTauri APIを呼び出す必要があります：

- **Tauri IPC**: `@tauri-apps/api`を使用
- **非同期処理**: `async/await`を使用
- **エラーハンドリング**: try-catchでエラーを処理

### 4.3 UIコンポーネントの活用

Phase 3で実装したSolid.js版UIコンポーネントを活用：

- **Button**: アクションボタン
- **Input**: フォーム入力
- **Modal**: ダイアログ表示
- **Toast**: 通知表示
- **Card**: カードレイアウト
- **Badge**: バッジ表示

---

## 5. 工数見積もり（概算）

| フェーズ | 内容 | 見積もり | ステータス |
| -------- | ---- | -------- | ---------- |
| Phase 1 | 調査・分析 | 1日 | 未着手 |
| Phase 2 | 優先度高のコンポーネント移行 | 3-5日 | 未着手 |
| Phase 3 | 優先度中のコンポーネント移行 | 2-3日 | 未着手 |
| Phase 4 | 優先度低のコンポーネント移行 | 1-2日 | 未着手 |
| Phase 5 | 統合テスト・ドキュメント更新 | 1日 | 未着手 |
| **合計** | | **8-12日** | **未着手** |

**注意**: 実際の工数は調査結果に基づいて調整が必要です。

---

## 6. 完了条件

- [ ] 全機能コンポーネントがSolid.jsで実装されている
- [ ] 各コンポーネントに.spec.mdが存在する
- [ ] TypeScriptの型が正しく定義されている
- [ ] コンポーネントが独立してレンダリングできる
- [ ] Tauri APIとの連携が正常に動作する
- [ ] 実装計画の進捗状況を更新
- [ ] ARCHITECTURE.mdを更新
- [ ] 実装ログを作成

---

## 7. 次のステップ

Phase 4完了後：

1. **Phase 5**: Leptos版の完全削除

---

## 8. 参考資料

- [Solid.js Documentation](https://www.solidjs.com/)
- [Tauri API Documentation](https://tauri.app/v1/api/js/)
- Phase 3-1実装計画: `docs/03_plans/ui-components-migration/20251206_01_phase3-1-basic-ui-components-plan.md`
- Phase 3-2実装計画: `docs/03_plans/ui-components-migration/20251206_02_phase3-2-medium-priority-components-plan.md`
- Phase 3-3実装計画: `docs/03_plans/ui-components-migration/20251206_03_phase3-3-low-priority-components-plan.md`
- Issue: https://github.com/otomatty/development-tools/issues/129

