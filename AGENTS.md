あなたは優秀なAIコーディングエージェントです。
以下のLegible Architectureの原則に則って実装作業を行なってください。

---

## 概要

### このフレームワークの目的

このフレームワークは、「Legible Architecture」の**Concept（概念）**と**Synchronization（同期）**の原則を中核に据えます。以下の目的を達成します：

- **AIにとって理解しやすい**: 変更対象のConceptやSynchronizationという限定的なコンテキストのみを理解すれば、安全で精度の高いコード生成が可能
- **人間にとって把握しやすい**: システムの全体像が把握しやすく、変更の影響範囲が明確
- **変更に強い**: 関心事の分離により、一箇所の変更が他に波及しにくい構造

### ConceptとSynchronizationの基本概念

*   **Concept**: アプリケーションの機能単位。自己完結しており、自身の状態とそれを操作するロジックのみを持つ。他のConceptを直接参照しない。
*   **Synchronization**: 複数のConceptを連携させるためのルールセット。「何が起きたら(when)、何をする(then)」を記述する。状態を持たず、Concept間の仲介役に徹する。

---

## 基本原則

### Legible Architectureの原則

1. **Conceptの独立性**: Conceptは、他のいかなるConceptも直接`use`してはならない。完全に独立した単位として設計する。
2. **状態の単一所有**: 状態は必ず単一のConceptに属する。Synchronizationは状態を持たない。
3. **Synchronizationの仲介役**: Concept間の連携は、Synchronizationを通じてのみ行われる。Synchronizationは状態を持たず、Conceptの状態を読み取り、別のConceptのアクションをトリガーする役割に徹する。

### なぜこの原則が重要なのか

- **AIの理解範囲を限定**: AIは変更対象のConceptやSynchronizationという限定的なコンテキストのみを理解すればよい
- **影響範囲の明確化**: 依存関係が明確なため、変更の影響範囲が即座に判定できる
- **テストの容易さ**: Concept単体のテストが容易で、Synchronizationのテストも複数のConceptの状態をインプットとして検証できる

---

## ディレクトリ構造

### 基本構造 (Rust Modules)

関連ファイルを機能単位（Concept）でまとめることで、関心事の分離を徹底します。

```plaintext
.
├── docs/
│   ├── 00_prompts/              # AI・開発者への指示書
│   ├── 01_issues/               # 問題・要件定義
│   │   ├── open/                # 未解決のIssue
│   │   └── resolved/            # 解決済みのIssue
│   ├── 02_research/             # 技術選定・調査結果
│   ├── 03_plans/                # 段階的な実装計画
│   └── 05_logs/                 # 作業記録
│
├── src/
│   ├── concepts/
│   │   ├── mod.rs
│   │   ├── post/
│   │   │   ├── mod.rs           # Post Conceptの公開API (モジュール定義)
│   │   │   ├── state.rs         # 状態の型定義と初期状態
│   │   │   ├── actions.rs       # 状態を操作するロジック (純粋関数)
│   │   │   ├── tests.rs         # 単体テスト (cargo test)
│   │   │   └── post.spec.md     # 仕様書 (このConceptの責務、状態、アクション、テストケースを記述)
│   │   │
│   │   └── user/
│   │       ├── mod.rs
│   │       ├── state.rs
│   │       ├── actions.rs
│   │       ├── tests.rs
│   │       └── user.spec.md
│   │
│   ├── synchronizations/
│   │   ├── mod.rs
│   │   ├── user_posts_sync.rs   # UserとPostを連携させるロジック
│   │   ├── user_posts_sync_test.rs
│   │   └── user_posts.spec.md   # この連携の目的やルール、テストケースを記述
│   │
│   ├── lib.rs                   # ライブラリのエントリーポイント
│   └── main.rs                  # アプリケーションのエントリーポイント
│
└── Cargo.toml                   # Rustのパッケージ定義
```

### 発展的な構造（ドメイン別グルーピング）

Conceptが増えてきたら、ドメイン（関連領域）でグルーピングします。

```plaintext
src/
├── concepts/
│   ├── identity/         # 認証・認可ドメイン
│   │   ├── user/
│   │   └── auth/
│   │
│   └── publishing/       # コンテンツ公開ドメイン
│       ├── post/
│       ├── comment/
│       └── like/
│
└── synchronizations/
    ├── identity_publishing_sync.rs # 認証と公開ドメイン間の連携
    └── publishing_internal_sync.rs # 公開ドメイン内部での連携
```

---

## コーディングルール

### A. Conceptのルール

#### 1. 独立性の原則

Conceptは、他のいかなるConceptも直接`use`してはならない。

**❌ 悪い例:**
```rust
// src/concepts/post/actions.rs
use crate::concepts::user::state::UserState; // ❌ 他のConceptを直接参照

pub fn add_post(state: PostState, post: Post, user_state: UserState) -> PostState {
    // ...
}
```

**✅ 良い例:**
```rust
// src/concepts/post/actions.rs
use super::state::{Post, PostState};

// 外部のConceptに依存せず、渡されたデータで自身の状態を更新するだけ
pub fn add_post(state: PostState, post: Post) -> PostState {
    let mut new_posts = state.posts;
    new_posts.push(post);
    PostState {
        posts: new_posts,
        ..state
    }
}
```

#### 2. 自己完結

Conceptは、自身の`state`とそれを操作する`actions`のみで構成される。`actions`は可能な限り純粋関数として実装する。

**ファイル構成:**
- `state.rs`: 状態の型定義と初期状態
- `actions.rs`: 状態を操作するロジック（純粋関数）
- `mod.rs`: 公開API（型定義、アクションの再エクスポート）
- `tests.rs`: 単体テスト（.spec.mdのTest Casesに基づいて記述）
- `{concept-name}.spec.md`: 仕様書（RequirementsとTest Casesを含む）

#### 3. 副作用の分離

外部APIの呼び出しやデータベースへのアクセスといった副作用はConcept内に直接記述しない。これらは上位層（`synchronizations`やアプリケーションサービス層）が担当し、結果を引数として`actions`に渡す。

**例: `src/concepts/post/actions.rs`**
```rust
use super::state::{Post, PostState};

// 外部のConceptに依存せず、渡されたデータで自身の状態を更新するだけ
pub fn add_post(state: PostState, post: Post) -> PostState {
    let mut new_posts = state.posts;
    new_posts.push(post);
    PostState {
        posts: new_posts,
        ..state
    }
}

// ❌ 副作用を含む実装は避ける
// pub async fn fetch_and_add_post(state: PostState, post_id: String) -> PostState {
//   const post = await fetch_post_from_api(post_id); // ❌ 副作用
//   add_post(state, post)
// }
```

### B. Synchronizationのルール

#### 1. 唯一の連携点

複数のConceptを`use`し、連携させることができるのは`synchronizations`ディレクトリ内のファイルのみ。

**例: `src/synchronizations/user_posts_sync.rs`**
```rust
use crate::concepts::user;
use crate::concepts::post;

// when: ユーザーが作成されたら
// then: そのユーザーの初期投稿を作成する
pub fn on_user_created(
    user_state: &user::state::UserState,
    post_state: post::state::PostState,
    user_id: &str
) -> post::state::PostState {
    let user = user_state.users.iter().find(|u| u.id == user_id);
    
    match user {
        None => post_state, // where: ユーザーが存在しない場合は何もしない
        Some(u) => {
            let welcome_post = post::state::Post {
                id: "post-0".to_string(),
                author_id: user_id.to_string(),
                content: format!("ようこそ、{}さん！", u.name),
            };
            post::actions::add_post(post_state, welcome_post)
        }
    }
}
```

#### 2. 状態を持たない

Synchronizationは状態を持たない。Conceptの状態を読み取り、別のConceptのアクションをトリガーする役割に徹する。

#### 3. 宣言的な記述

「when (イベント), where (条件), then (アクション)」を意識したコードを記述する。イベント駆動の設計が適している。

**命名規則:**
- `on_{event}`: イベントハンドラー（例: `on_user_created`）
- `when_{condition}`: 条件付き処理（例: `when_user_likes_post`）

### C. テストとドキュメント

#### 1. テスト駆動開発

- 新しい`action`を実装する前に、必ず`tests.rs`にテストケースを追加する
- `cargo test`で、Concept単体のロジックが期待通りに動作することを保証する
- Synchronizationのテストでは、複数のConceptの状態をインプットとし、連携後の状態が期待通りか検証する

**例: `src/concepts/post/tests.rs`**
```rust
#[cfg(test)]
mod tests {
    use super::actions::*;
    use super::state::*;

    #[test]
    fn test_add_post() {
        let state = PostState::default();
        let new_post = Post {
            id: "1".to_string(),
            author_id: "user-1".to_string(),
            content: "Hello".to_string(),
        };
        
        let result = add_post(state, new_post.clone());
        
        assert_eq!(result.posts.len(), 1);
        assert_eq!(result.posts[0], new_post);
    }
}
```

#### 2. ドキュメント駆動開発

このフレームワークでは、**ドキュメント駆動開発**と**テスト駆動開発**を組み合わせて実践します。

**ドキュメントの種類:**

1. **Spec（仕様書）**: `src/concepts/{concept-name}/{concept-name}.spec.md` / `src/synchronizations/{name}.spec.md`
   - Concept/Synchronizationの仕様定義とテストケース定義を記述
   - Requirements（要件）とTest Cases（テストケース）を含む
   - 実装ファイルと同じディレクトリに配置

2. **Issue（問題・要件）**: `docs/01_issues/open/YYYY_MM/YYYYMMDD_{issue-name}.md`
   - 新機能の要件定義やバグ報告を記述
   - 実装前に作成し、実装完了後に`resolved/`に移動

3. **Research（技術調査）**: `docs/02_research/YYYY_MM/YYYYMMDD_{research-name}.md`
   - 技術選定や調査結果を記述
   - Concept/Synchronizationの実装前に技術的な検討が必要な場合に作成

4. **Plan（実装計画）**: `docs/03_plans/{feature-name}/YYYYMMDD_{plan-name}.md`
   - 段階的な実装計画を記述
   - 複数のConcept/Synchronizationにまたがる機能の場合に作成

5. **Log（作業ログ）**: `docs/05_logs/YYYY_MM/YYYYMMDD/{log-name}.md`
   - 実装作業の記録を記述
   - 実装完了後に作成

**仕様書の配置:**

- **Conceptの仕様書**: `src/concepts/{concept-name}/{concept-name}.spec.md`
  - そのConceptが担当する責務、管理する状態の構造、各アクションの目的と動作を記述
  - Requirements（要件）とTest Cases（テストケース）を含む

- **Synchronizationの仕様書**: `src/synchronizations/{name}.spec.md`
  - どのConcept間を、どのようなルールで連携させるのかを記述
  - when（イベント）、where（条件）、then（アクション）を明確に記述
  - Requirements（要件）とTest Cases（テストケース）を含む

**仕様書の構造例:**

```markdown
# Post Concept Specification

## Related Files

- Implementation: `src/concepts/post/mod.rs`
- State: `src/concepts/post/state.rs`
- Actions: `src/concepts/post/actions.rs`
- Tests: `src/concepts/post/tests.rs`

## Related Documentation

- Issue: `docs/01_issues/resolved/2025_10/20251022_01_post-concept.md`
- Plan: `docs/03_plans/post-feature/20251022_01_implementation-plan.md`
- Log: `docs/05_logs/2025_10/20251022/post-concept-implementation.md`
- Synchronizations:
  - user_posts_sync: `src/synchronizations/user_posts_sync.rs`

## Requirements

### 責務
- 投稿データの管理
- 投稿の追加・更新・削除

### 状態構造
- PostState: { posts: Vec<Post> }
- Post: { id: String, author_id: String, content: String }

### アクション
- add_post: 投稿を追加
- update_post: 投稿を更新
- delete_post: 投稿を削除

## Test Cases

### TC-001: add_post
- Given: 空のPostState
- When: add_post(state, new_post)を実行
- Then: posts配列にnew_postが追加される

### TC-002: update_post
- Given: 既存の投稿を含むPostState
- When: update_post(state, post_id, updated_content)を実行
- Then: 指定された投稿のcontentが更新される
```

**ファイルとドキュメントの同期:**

実装ファイルを修正したら、必ず関連ドキュメントも更新してください：

**必須の更新:**
- `.spec.md`の「Requirements」セクションを更新
- `.spec.md`の「Test Cases」に新規テストケースを追加
- `tests.rs`に対応するテストを追加（.spec.mdのTest Casesに基づいて）
- DEPENDENCY MAPを更新（実装ファイルの先頭コメント）

**推奨の更新:**
- 実装計画の進捗状況を更新（Planが存在する場合）
- 作業ログを記録（大きな変更の場合）
- Issueの状態を更新（open → resolved など、バグ修正の場合）

**更新チェックリスト:**
- [ ] `.spec.md`の「Requirements」は最新か？
- [ ] `.spec.md`の「Test Cases」に新しいケースを追加したか？
- [ ] `tests.rs`は`.spec.md`のTest Casesと一致しているか？
- [ ] DEPENDENCY MAPが最新か？（Parents / Dependencies）
- [ ] 親ファイル（Parents）のDEPENDENCY MAPも更新したか？
- [ ] Spec/Issue/Plan/Logへの参照が最新か？

---

## 実装フロー

### ドキュメント駆動開発のワークフロー

```
PROMPT → ISSUE → RESEARCH → PLAN → SPEC+TEST → IMPLEMENTATION → LOG
```

1. **Prompt（プロンプト）**: `docs/00_prompts/` - AI・開発者への指示書（このファイル）
2. **Issue（問題・要件）**: `docs/01_issues/open/YYYY_MM/` - 問題・要件定義
3. **Research（技術調査）**: `docs/02_research/YYYY_MM/` - 技術選定・調査結果（必要に応じて）
4. **Plan（実装計画）**: `docs/03_plans/{機能名}/` - 段階的な実装計画（複数Conceptにまたがる場合）
5. **Spec（仕様書）**: `src/concepts/{concept-name}/{concept-name}.spec.md` / `src/synchronizations/{name}.spec.md` - 仕様定義＋テストケース定義
6. **Implementation（実装）**: テスト駆動開発で実装
7. **Log（作業ログ）**: `docs/05_logs/YYYY_MM/YYYYMMDD/` - 作業記録

### Concept作成フロー

```
1. Issue作成（必要に応じて）
   └── docs/01_issues/open/YYYY_MM/YYYYMMDD_{concept-name}.md
       - 新Conceptの要件を定義

2. Research作成（技術的な検討が必要な場合）
   └── docs/02_research/YYYY_MM/YYYYMMDD_{concept-name}-research.md
       - 技術選定や設計方針を検討

3. Plan作成（複数のConcept/Synchronizationにまたがる場合）
   └── docs/03_plans/{feature-name}/YYYYMMDD_{plan-name}.md
       - 段階的な実装計画を記述

4. Spec作成（仕様書）
   └── src/concepts/{concept-name}/{concept-name}.spec.md
       - Requirements（要件）を記述
       - Test Cases（テストケース）を記述

5. ディレクトリ作成
   └── src/concepts/{concept-name}/

6. ファイル作成
   ├── state.rs      # 状態の型定義と初期状態
   ├── actions.rs    # 状態を操作するロジック
   ├── mod.rs        # 公開API
   └── tests.rs      # テスト

7. テスト駆動開発
   ├── .spec.mdのTest Casesに基づいてテストケースを書く
   ├── テストを実行（失敗することを確認）
   ├── 実装する（テストが通る最小限の実装）
   ├── リファクタリング
   └── cargo test で確認（成功することを確認）

8. ドキュメント更新
   ├── .spec.mdのRequirementsを実装に合わせて更新
   └── DEPENDENCY MAPを記載

9. Log作成
   └── docs/05_logs/YYYY_MM/YYYYMMDD/{concept-name}-implementation.md
       - 実装作業の記録

10. Issue解決
    └── docs/01_issues/open/ → docs/01_issues/resolved/ へ移動
```

### Synchronization作成フロー

```
1. Issue作成（必要に応じて）
   └── docs/01_issues/open/YYYY_MM/YYYYMMDD_{sync-name}.md
       - 新Synchronizationの要件を定義

2. Research作成（技術的な検討が必要な場合）
   └── docs/02_research/YYYY_MM/YYYYMMDD_{sync-name}-research.md
       - 連携方法の検討

3. Plan作成（複数のConcept/Synchronizationにまたがる場合）
   └── docs/03_plans/{feature-name}/YYYYMMDD_{plan-name}.md
       - 段階的な実装計画を記述

4. Spec作成（仕様書）
   └── src/synchronizations/{name}.spec.md
       - Requirements（要件）を記述
       - when（イベント）、where（条件）、then（アクション）を明確に記述
       - Test Cases（テストケース）を記述

5. ファイル作成
   ├── src/synchronizations/{name}_sync.rs
   └── src/synchronizations/{name}_sync_test.rs

6. 連携するConceptを特定
   └── どのConcept間を連携させるか明確にする

7. テスト駆動開発
   ├── .spec.mdのTest Casesに基づいてテストケースを書く
   ├── 複数のConceptの状態をインプットとしてテスト
   ├── テストを実行（失敗することを確認）
   ├── 実装する（テストが通る最小限の実装）
   ├── リファクタリング
   └── cargo test で確認（成功することを確認）

8. ドキュメント更新
   ├── .spec.mdのRequirementsを実装に合わせて更新
   └── DEPENDENCY MAPを記載

9. Log作成
   └── docs/05_logs/YYYY_MM/YYYYMMDD/{sync-name}-implementation.md
       - 実装作業の記録

10. Issue解決
    └── docs/01_issues/open/ → docs/01_issues/resolved/ へ移動
```

### テスト駆動開発の流れ

```
1. SpecのTest Casesを確認
   └── .spec.mdのTest Casesセクションを参照

2. テストケースを書く
   └── tests.rsに.spec.mdのTest Casesに基づいてテストコードを記述

3. テストを実行（失敗することを確認）
   └── cargo test

4. 実装する
   └── テストが通る最小限の実装

5. リファクタリング
   └── コードの品質を向上させる

6. テストを再実行（成功することを確認）
   └── cargo test

7. Specの更新
   └── 実装に合わせて.spec.mdのRequirementsを更新
```

---

## 依存関係の明示

### Concept間の依存関係マッピング

Conceptは他のConceptを直接参照しないため、Concept間の依存関係は存在しません。代わりに、SynchronizationがConcept間の連携を担当します。

### Synchronizationの依存関係

Synchronizationファイルの先頭に、依存関係マップを記載してください：

```rust
/**
 * UserPosts Synchronization
 *
 * DEPENDENCY MAP:
 * Concepts (Concept files that this synchronization imports):
 *   ├─ src/concepts/user/mod.rs
 *   └─ src/concepts/post/mod.rs
 * Related Documentation:
 *   ├─ Spec: ./user_posts.spec.md
 *   ├─ Tests: ./user_posts_sync_test.rs
 *   ├─ Related Concepts:
 *   │   ├─ User: src/concepts/user/user.spec.md
 *   │   └─ Post: src/concepts/post/post.spec.md
 *   ├─ Issue: docs/01_issues/resolved/2025_10/20251022_01_user-posts-sync.md
 *   ├─ Plan: docs/03_plans/user-posts-feature/20251022_01_implementation-plan.md
 *   └─ Log: docs/05_logs/2025_10/20251022/user-posts-sync-implementation.md
 */

use crate::concepts::{user, post};

// ...
```

### Conceptファイルの依存関係マッピング

Conceptファイルの先頭にも、依存関係マップを記載してください：

```rust
/**
 * Post Concept
 *
 * DEPENDENCY MAP:
 *
 * Parents (Files that import this Concept):
 *   ├─ src/synchronizations/user_posts_sync.rs
 *   ├─ src/ui/components/post_list.rs
 *   └─ src/ui/pages/post_page.rs
 * Related Documentation:
 *   ├─ Spec: ./post.spec.md
 *   ├─ Tests: ./tests.rs
 *   ├─ Synchronizations:
 *   │   └─ user_posts_sync: src/synchronizations/user_posts_sync.rs
 *   ├─ Issue: docs/01_issues/resolved/2025_10/20251022_01_post-concept.md
 *   ├─ Plan: docs/03_plans/post-feature/20251022_01_implementation-plan.md
 *   └─ Log: docs/05_logs/2025_10/20251022/post-concept-implementation.md
 */

pub mod state;
pub mod actions;
#[cfg(test)]
mod tests;

pub use state::*;
pub use actions::*;
// ...
```

**なぜ必要か:**
- 修正時の影響範囲が即座に判定できる
- リファクタリングのリスク評価が容易
- 軽量AIモデルでも依存関係を理解できる
- デッドコード検出が簡単

---

## チェックリスト

### Concept作成時のチェックリスト

新しいConceptを作成する際は、以下を確認してください：

**ドキュメント作成:**
- [ ] Issueを作成したか？（必要に応じて）`docs/01_issues/open/YYYY_MM/YYYYMMDD_{concept-name}.md`
- [ ] Researchを作成したか？（技術的な検討が必要な場合）`docs/02_research/YYYY_MM/YYYYMMDD_{concept-name}-research.md`
- [ ] Planを作成したか？（複数のConcept/Synchronizationにまたがる場合）`docs/03_plans/{feature-name}/YYYYMMDD_{plan-name}.md`
- [ ] Specを作成したか？`src/concepts/{concept-name}/{concept-name}.spec.md`
  - [ ] Requirements（要件）を記述したか？
  - [ ] Test Cases（テストケース）を記述したか？

**実装:**
- [ ] `src/concepts/{concept-name}/` ディレクトリを作成したか？
- [ ] `state.rs` に状態の型定義と初期状態を定義したか？
- [ ] `actions.rs` に純粋関数としてアクションを実装したか？
- [ ] `mod.rs` に公開APIを定義したか？
- [ ] 他のConceptを直接`use`していないか？（独立性の原則）
- [ ] 副作用（API呼び出し、DBアクセス）を含んでいないか？

**テスト:**
- [ ] `.spec.md`のTest Casesに基づいて`tests.rs`にテストケースを追加したか？
- [ ] `cargo test` でテストが通るか？

**ドキュメント更新:**
- [ ] `.spec.md`のRequirementsを実装に合わせて更新したか？
- [ ] ファイル先頭に DEPENDENCY MAP を記載したか？
  - [ ] Specへの参照を記載したか？
  - [ ] Issue/Plan/Logへの参照を記載したか？（存在する場合）

**完了作業:**
- [ ] Logを作成したか？`docs/05_logs/YYYY_MM/YYYYMMDD/{concept-name}-implementation.md`
- [ ] Issueを`resolved/`に移動したか？（Issueを作成した場合）

### Synchronization作成時のチェックリスト

新しいSynchronizationを作成する際は、以下を確認してください：

**ドキュメント作成:**
- [ ] Issueを作成したか？（必要に応じて）`docs/01_issues/open/YYYY_MM/YYYYMMDD_{sync-name}.md`
- [ ] Researchを作成したか？（技術的な検討が必要な場合）`docs/02_research/YYYY_MM/YYYYMMDD_{sync-name}-research.md`
- [ ] Planを作成したか？（複数のConcept/Synchronizationにまたがる場合）`docs/03_plans/{feature-name}/YYYYMMDD_{plan-name}.md`
- [ ] Specを作成したか？`src/synchronizations/{name}.spec.md`
  - [ ] Requirements（要件）を記述したか？
  - [ ] when（イベント）、where（条件）、then（アクション）を明確に記述したか？
  - [ ] Test Cases（テストケース）を記述したか？

**実装:**
- [ ] `src/synchronizations/{name}_sync.rs` を作成したか？
- [ ] 連携するConceptを明確に特定したか？
- [ ] 「when (イベント), where (条件), then (アクション)」を意識した実装か？
- [ ] 状態を持たない実装か？（純粋関数として実装）

**テスト:**
- [ ] `.spec.md`のTest Casesに基づいて`{name}_sync_test.rs`（またはモジュール内テスト）にテストケースを追加したか？
- [ ] 複数のConceptの状態をインプットとしてテストしているか？
- [ ] `cargo test` でテストが通るか？

**ドキュメント更新:**
- [ ] `.spec.md`のRequirementsを実装に合わせて更新したか？
- [ ] ファイル先頭に DEPENDENCY MAP を記載したか？
  - [ ] Specへの参照を記載したか？
  - [ ] Issue/Plan/Logへの参照を記載したか？（存在する場合）

**完了作業:**
- [ ] Logを作成したか？`docs/05_logs/YYYY_MM/YYYYMMDD/{sync-name}-implementation.md`
- [ ] Issueを`resolved/`に移動したか？（Issueを作成した場合）

### コード更新時のチェックリスト

既存のConceptやSynchronizationを更新する際は、以下を確認してください：

**ドキュメント更新:**
- [ ] `.spec.md`の「Requirements」セクションを更新したか？
- [ ] `.spec.md`の「Test Cases」に新規テストケースを追加したか？
- [ ] 実装計画の進捗状況を更新したか？（Planが存在する場合）
- [ ] 作業ログを記録したか？（大きな変更の場合）

**テスト:**
- [ ] `tests.rs`に`.spec.md`のTest Casesに基づいてテストケースを追加・更新したか？
- [ ] `cargo test` でテストが通るか？

**依存関係:**
- [ ] DEPENDENCY MAP が最新か？（Parents / Dependencies）
- [ ] 親ファイル（Parents）の DEPENDENCY MAP も更新したか？
- [ ] Spec/Issue/Plan/Logへの参照が最新か？

**原則の確認:**
- [ ] 他のConceptを直接`use`していないか？（Conceptの場合）
- [ ] 副作用を含んでいないか？（Conceptの場合）
- [ ] 状態を持たない実装か？（Synchronizationの場合）

**Issue管理:**
- [ ] Issueの状態を更新したか？（open → resolved など、バグ修正の場合）

---

## AIモデルへの指示例

### Concept追加の指示例

```
【タスク】Post Conceptに「いいね」機能を追加

【参照ドキュメント】
- 仕様書: src/concepts/post/post.spec.md
- Issue: docs/01_issues/open/2025_10/20251022_01_post-like-feature.md
- Plan: docs/03_plans/post-feature/20251022_01_like-implementation-plan.md（存在する場合）
- 依存関係: src/concepts/post/actions.rs の DEPENDENCY MAP コメント

【更新するファイル】
1. src/concepts/post/post.spec.md
   - Requirementsセクションにいいね機能の要件を追加
   - Test Casesセクションにincrement_like/decrement_likeのテストケースを追加

2. src/concepts/post/state.rs
   - Post構造体にlikes_countフィールドを追加
   - PostStateにlikes_countの集計ロジックを追加
   - DEPENDENCY MAP の確認（変更があれば更新）

3. src/concepts/post/actions.rs
   - increment_like アクションを追加
   - decrement_like アクションを追加
   - DEPENDENCY MAP の確認（変更があれば更新）

4. src/concepts/post/tests.rs
   - .spec.mdのTest Casesに基づいてincrement_likeのテストケース追加
   - .spec.mdのTest Casesに基づいてdecrement_likeのテストケース追加

5. src/concepts/post/mod.rs
   - increment_like/decrement_like を再エクスポート

6. docs/05_logs/2025_10/20251022/post-like-feature-implementation.md
   - 実装作業の記録を作成

【依存関係チェック】
- Parents（このConceptを使用）: user_posts_sync.rs, post_list.rs
  - これらのファイルのDEPENDENCY MAPも更新が必要か確認
- Dependencies（このConceptが使用）: state.rs, actions.rs

【重要な原則】
- 他のConceptを直接useしない
- 副作用を含まない純粋関数として実装
- .spec.mdのTest Casesに基づいてテスト駆動開発で実装
- 実装完了後、.spec.mdのRequirementsを実装に合わせて更新
```

### Synchronization追加の指示例

```
【タスク】ユーザーが投稿にいいねしたら通知を送るSynchronizationを追加

【参照ドキュメント】
- User Concept: src/concepts/user/user.spec.md
- Post Concept: src/concepts/post/post.spec.md
- Notification Concept: src/concepts/notification/notification.spec.md
- Issue: docs/01_issues/open/2025_10/20251022_01_user-like-notification.md（存在する場合）
- Plan: docs/03_plans/notification-feature/20251022_01_implementation-plan.md（存在する場合）

【作成するファイル】
1. src/synchronizations/user_like_post_notification.spec.md
   - Requirementsセクションに連携の要件を記述
   - when（イベント）、where（条件）、then（アクション）を明確に記述
   - Test Casesセクションにテストケースを記述

2. src/synchronizations/user_like_post_notification_sync.rs
   - User, Post, Notification Conceptを連携
   - when: ユーザーが投稿にいいねしたら
   - then: 通知を作成する
   - DEPENDENCY MAP を記載（Spec/Issue/Planへの参照を含む）

3. src/synchronizations/user_like_post_notification_sync_test.rs (またはモジュール内)
   - .spec.mdのTest Casesに基づいてテストケースを記述
   - User, Post, Notification の状態をインプットとしてテスト
   - 連携後の状態が期待通りか検証

4. docs/05_logs/2025_10/20251022/user-like-notification-implementation.md
   - 実装作業の記録を作成

【依存関係チェック】
- Concepts（このSynchronizationが使用）:
  - User: src/concepts/user/mod.rs
  - Post: src/concepts/post/mod.rs
  - Notification: src/concepts/notification/mod.rs
- これらのConceptのDEPENDENCY MAPにこのSynchronizationへの参照を追加

【重要な原則】
- 状態を持たない純粋関数として実装
- 「when, where, then」を意識した宣言的な記述
- .spec.mdのTest Casesに基づいてテスト駆動開発で実装
- 実装完了後、.spec.mdのRequirementsを実装に合わせて更新
```

### バグ修正の指示例

```
【タスク】Post Conceptのadd_postアクションで、重複チェックが不十分なバグを修正

【参照ドキュメント】
- 仕様書: src/concepts/post/post.spec.md
- Issue: docs/01_issues/open/2025_10/20251022_01_post-duplicate-bug.md
- 依存関係: src/concepts/post/actions.rs の DEPENDENCY MAP コメント

【更新するファイル】
1. src/concepts/post/post.spec.md
   - Requirementsセクションに重複チェックの要件を追加
   - Test Casesセクションに重複チェックのテストケースを追加（バグを再現するケースを含む）

2. src/concepts/post/actions.rs
   - add_post アクションに重複チェックを追加
   - DEPENDENCY MAP の確認（変更があれば更新）

3. src/concepts/post/tests.rs
   - .spec.mdのTest Casesに基づいて重複チェックのテストケース追加
   - 既存のテストが壊れていないか確認

4. docs/05_logs/2025_10/20251022/post-duplicate-bugfix.md
   - バグ修正作業の記録を作成

5. docs/01_issues/open/2025_10/20251022_01_post-duplicate-bug.md
   - Issueをresolved/に移動

【依存関係チェック】
- Parents（このConceptを使用）: user_posts_sync.rs, post_list.rs
- 影響を受ける可能性のあるファイルを確認
  - これらのファイルのDEPENDENCY MAPも更新が必要か確認

【重要な原則】
- 既存のテストが通ることを確認
- 他のConceptに影響を与えない
- 副作用を含まない純粋関数として実装
- .spec.mdのTest Casesに基づいてテスト駆動開発で実装
- 実装完了後、.spec.mdのRequirementsを実装に合わせて更新
```

---

## 発展的な質問への回答

### Q1: 3個以上のConceptが関わる場合はどうするのでしょうか？

**回答:** 1つのSynchronizationファイルで3つ以上のConceptを扱います。重要なのは、その**連携の関心事が単一であること**です。

*   **命名**: ファイル名は、連携の目的がわかるように命名します。
    *   例: `ユーザーが投稿にいいねしたら通知を送る` という連携
        *   関わるConcept: `User`, `Post`, `Notification`
        *   ファイル名: `user_like_post_notification_sync.rs`
*   **責務**: 1つのSynchronizationファイルが、1つのビジネス上のユースケースに対応するように設計します。もし1つのファイルが肥大化し、複数のユースケースを扱っている場合は、ファイルを分割することを検討してください。

**例: `src/synchronizations/user_like_post_notification_sync.rs`**
```rust
use crate::concepts::{user, post, notification};

// when: ユーザーが投稿にいいねしたら
// then: 通知を作成する
pub fn on_user_likes_post(
    user_state: &user::state::UserState,
    post_state: &post::state::PostState,
    notification_state: notification::state::NotificationState,
    user_id: &str,
    post_id: &str
) -> notification::state::NotificationState {
    let user = user_state.users.iter().find(|u| u.id == user_id);
    let post = post_state.posts.iter().find(|p| p.id == post_id);
    
    if user.is_none() || post.is_none() {
        return notification_state; // where: ユーザーまたは投稿が存在しない場合は何もしない
    }
    
    let u = user.unwrap();
    let p = post.unwrap();

    let notification = notification::state::Notification {
        id: format!("notification-{}", 12345), // Timestamp mockup
        user_id: p.author_id.clone(),
        message: format!("{}さんがあなたの投稿にいいねしました", u.name),
    };

    notification::actions::add_notification(notification_state, notification)
}
```

### Q2: 実装が複雑になってきた時にはどのようなディレクトリ構造が望ましいのでしょうか？

**回答:** Conceptが増えてきたら、ドメイン（関連領域）でグルーピングします。

```plaintext
src/
├── concepts/
│   ├── identity/         # 認証・認可ドメイン
│   │   ├── user/
│   │   │   ├── mod.rs
│   │   │   ├── state.rs
│   │   │   ├── actions.rs
│   │   │   ├── tests.rs
│   │   │   └── user.spec.md
│   │   └── auth/
│   │       ├── mod.rs
│   │       ├── state.rs
│   │       ├── actions.rs
│   │       ├── tests.rs
│   │       └── auth.spec.md
│   │
│   └── publishing/       # コンテンツ公開ドメイン
│       ├── post/
│       │   ├── mod.rs
│       │   ├── state.rs
│       │   ├── actions.rs
│       │   ├── tests.rs
│       │   └── post.spec.md
│       ├── comment/
│       │   ├── mod.rs
│       │   ├── state.rs
│       │   ├── actions.rs
│       │   ├── tests.rs
│       │   └── comment.spec.md
│       └── like/
│           ├── mod.rs
│           ├── state.rs
│           ├── actions.rs
│           ├── tests.rs
│           └── like.spec.md
│
└── synchronizations/
    ├── identity_publishing_sync.rs # 認証と公開ドメイン間の連携
    ├── identity_publishing_sync_test.rs
    ├── identity_publishing.spec.md
    ├── publishing_internal_sync.rs # 公開ドメイン内部での連携
    ├── publishing_internal_sync_test.rs
    └── publishing_internal.spec.md
```

### Q3: Concept間にまたがりそうな状態などはどうしたら良いのでしょうか？

**回答:** 「Legible Architecture」の原則では、**状態は必ず単一のConceptに属します**。一見またがっているように見える状態は、以下のいずれかの方法で解決します。

1.  **主となるConceptに所属させる**:
    *   例: 「ユーザーがいいねした投稿のリスト」
    *   これは `User` Conceptの状態（`liked_post_ids: Vec<String>`）として持つのが自然かもしれません。`Post` Conceptは自身のいいね数をカウントするだけです。

    **例: `src/concepts/user/state.rs`**
    ```rust
    pub struct User {
        pub id: String,
        pub name: String,
        pub liked_post_ids: Vec<String>, // ユーザーがいいねした投稿のIDリスト
    }
    ```

2.  **新しいConceptを作成する**:
    *   状態の所有者が曖昧で、どちらのConceptに置いても不自然な場合は、その状態を管理するための新しいConceptを作成します。
    *   例: 「ユーザーと投稿のマッチング情報」
    *   `User` にも `Post` にも属しづらい場合、`Matching` という新しいConceptを作成し、`{ user_id, post_id, score }` のような状態を管理させます。

    **例: `src/concepts/matching/state.rs`**
    ```rust
    pub struct Matching {
        pub user_id: String,
        pub post_id: String,
        pub score: i32,
    }

    pub struct MatchingState {
        pub matchings: Vec<Matching>,
    }
    ```

**重要なのは、Synchronizationは状態を持たず、必ずいずれかのConceptが状態の「信頼できる情報源（Single Source of Truth）」となるルールを徹底することです。**

---

## まとめ

このフレームワークを採用することで、以下のメリットが得られます：

- **AIにとって理解しやすい**: 変更対象のConceptやSynchronizationという限定的なコンテキストのみを理解すれば、安全で精度の高いコード生成が可能。ドキュメント駆動開発により、仕様書（.spec.md）から要件とテストケースを明確に把握できる
- **人間にとって把握しやすい**: システムの全体像が把握しやすく、変更の影響範囲が明確。Issue/Research/Plan/Logにより、開発の文脈と意思決定の経緯が追跡可能
- **変更に強い**: 関心事の分離により、一箇所の変更が他に波及しにくい構造。DEPENDENCY MAPにより、影響範囲が即座に判定できる
- **テストが容易**: Concept単体のテストが容易で、Synchronizationのテストも複数のConceptの状態をインプットとして検証できる。.spec.mdのTest Casesに基づいてテストを記述することで、仕様とテストの整合性が保たれる
- **ドキュメントとコードの同期**: 実装ファイルと仕様書（.spec.md）が同じディレクトリに配置され、常に同期を保つことで、コードとドキュメントの乖離を防ぐ
- **開発プロセスの可視化**: Issue/Research/Plan/Logにより、開発プロセス全体が可視化され、後から振り返りや引き継ぎが容易

このガイドラインに従うことで、AIと人間双方にとって可読性が高く、変更に強いソフトウェア構造を実現できます。

