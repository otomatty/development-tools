# PRD: ホーム画面（GitHub連携ゲーミフィケーション）

**Document Version:** 1.0  
**Last Updated:** 2024-11-26  
**Status:** Draft  
**GitHub Issue:** [#4](https://github.com/otomatty/development-tools/issues/4)

---

## 1. 概要（Executive Summary）

### 1.1 プロダクトビジョン

開発者のモチベーション向上を目的とした、GitHubと連携するゲーミフィケーション機能付きホーム画面を実装する。GitHub上の活動データをリアルタイムで取得し、レベルシステム、バッジ、ストリーク、チャレンジなどのゲーミフィケーション要素を通じて、継続的な開発活動を促進する。

### 1.2 目標

| 目標 | 指標 | ターゲット |
|------|------|-----------|
| 開発継続率向上 | 週間アクティブ日数 | +20% |
| エンゲージメント | アプリ起動頻度 | 1日1回以上 |
| 達成感の提供 | バッジ取得数 | 月3個以上 |

### 1.3 対象ユーザー

- **プライマリ**: 個人開発者（OSS貢献者、学習者、趣味開発者）
- **セカンダリ**: 業務開発者（将来的にチーム機能追加予定）

---

## 2. 技術スタック

| レイヤー | 技術 | 理由 |
|---------|------|------|
| **フロントエンド** | Leptos (Rust/WASM) | 既存アプリとの統合、型安全性 |
| **バックエンド** | Tauri 2.0 + Rust | デスクトップアプリ、セキュリティ |
| **データベース** | SQLite | ローカル優先、オフライン対応 |
| **認証** | GitHub OAuth 2.0 | 標準的な認証フロー |
| **スタイリング** | Tailwind CSS | 既存スタイルとの統合 |

---

## 3. 機能要件

### 3.1 GitHub OAuth認証

#### 3.1.1 ユーザーストーリー

> 開発者として、GitHubアカウントでログインしたい。自分の活動データを安全に取得するために。

#### 3.1.2 機能詳細

| 機能 | 説明 | 優先度 |
|------|------|--------|
| OAuth認証フロー | GitHub OAuth Appを使用した認証 | P0 |
| カスタムプロトコル | `development-tools://callback` でのコールバック処理 | P0 |
| トークン保存 | アクセストークンの安全な保存（暗号化SQLite） | P0 |
| トークンリフレッシュ | 有効期限切れ時の自動リフレッシュ | P1 |
| ログアウト | トークン削除とセッションクリア | P0 |

#### 3.1.3 必要なスコープ

```
read:user       - ユーザープロフィール情報
repo            - リポジトリ情報（プライベート含む）
read:org        - Organization情報（将来のチーム機能用）
```

#### 3.1.4 シーケンス図

```
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
│  User   │     │  Tauri  │     │ Browser │     │ GitHub  │
└────┬────┘     └────┬────┘     └────┬────┘     └────┬────┘
     │               │               │               │
     │  Login Click  │               │               │
     │──────────────>│               │               │
     │               │  Open Auth URL│               │
     │               │──────────────>│               │
     │               │               │  OAuth Flow   │
     │               │               │──────────────>│
     │               │               │               │
     │               │               │<──────────────│
     │               │               │  Callback     │
     │               │  Custom Proto │               │
     │               │<──────────────│               │
     │               │               │               │
     │               │  Exchange Code for Token      │
     │               │─────────────────────────────>│
     │               │                               │
     │               │<─────────────────────────────│
     │               │  Access Token                 │
     │               │               │               │
     │  Success      │               │               │
     │<──────────────│               │               │
     │               │               │               │
```

---

### 3.2 GitHub メトリクス取得

#### 3.2.1 ユーザーストーリー

> 開発者として、自分のGitHub活動を数値で見たい。進捗を把握し、モチベーションを維持するために。

#### 3.2.2 取得データ一覧

| カテゴリ | メトリクス | API | 更新頻度 |
|---------|----------|-----|----------|
| **コントリビューション** | コミット数 | REST API | リアルタイム |
| | PR作成数 | REST API | リアルタイム |
| | PRマージ数 | REST API | リアルタイム |
| | Issue作成数 | REST API | リアルタイム |
| | Issue解決数 | REST API | リアルタイム |
| **活動量** | Contribution Graph | GraphQL API | 1時間 |
| | アクティブリポジトリ数 | REST API | 1日 |
| | 連続コミット日数 | 計算 | リアルタイム |
| **品質** | PRレビュー数 | REST API | リアルタイム |
| | PRマージ率 | 計算 | 1日 |
| | Issue解決率 | 計算 | 1日 |
| **成長** | 使用言語一覧 | REST API | 1日 |
| | スター獲得数 | REST API | 1時間 |
| | フォロワー数 | REST API | 1日 |

#### 3.2.3 GraphQL クエリ例（Contribution Graph）

```graphql
query($login: String!) {
  user(login: $login) {
    contributionsCollection {
      contributionCalendar {
        totalContributions
        weeks {
          contributionDays {
            contributionCount
            date
            weekday
          }
        }
      }
    }
  }
}
```

---

### 3.3 レベルシステム

#### 3.3.1 ユーザーストーリー

> 開発者として、活動に応じてレベルアップしたい。RPGのような達成感を得るために。

#### 3.3.2 経験値（XP）テーブル

| アクション | 獲得XP | 理由 |
|-----------|--------|------|
| コミット | +10 XP | 基本的な開発活動 |
| PR作成 | +30 XP | まとまった作業の完了 |
| PRマージ | +50 XP | 品質を満たした成果 |
| Issue作成 | +15 XP | 問題発見・提案 |
| Issue解決 | +40 XP | 問題解決 |
| PRレビュー | +25 XP | コラボレーション |
| スター獲得 | +5 XP | 他者からの評価 |
| 連続コミットボーナス（1日） | +20 XP | 継続の奨励 |

#### 3.3.3 レベル計算式

```rust
/// レベルに必要な累計XP（対数的成長）
/// 
/// Level 1: 0 XP
/// Level 10: 1,000 XP  
/// Level 25: 10,000 XP
/// Level 50: 50,000 XP
/// Level 100: 200,000 XP (MAX)
fn xp_for_level(level: u32) -> u32 {
    if level <= 1 {
        return 0;
    }
    // 累計XP = 50 * (level - 1)^2
    50 * (level - 1).pow(2)
}

fn level_from_xp(total_xp: u32) -> u32 {
    // XPからレベルを逆算
    let level = ((total_xp as f64 / 50.0).sqrt() + 1.0).floor() as u32;
    level.min(100) // 最高レベル100
}
```

#### 3.3.4 レベルテーブル

| レベル | 必要累計XP | 次レベルまで |
|--------|-----------|-------------|
| 1 | 0 | 50 |
| 5 | 800 | 450 |
| 10 | 4,050 | 950 |
| 25 | 28,800 | 2,450 |
| 50 | 120,050 | 4,950 |
| 75 | 273,800 | 7,450 |
| 100 (MAX) | 490,050 | - |

---

### 3.4 バッジシステム

#### 3.4.1 ユーザーストーリー

> 開発者として、特定の達成でバッジを獲得したい。コレクション要素を楽しみ、達成感を得るために。

#### 3.4.2 バッジカテゴリ

##### 🏅 マイルストーンバッジ

| ID | 名前 | 条件 | レア度 |
|----|------|------|--------|
| `first_blood` | First Blood | 初コミット | Bronze |
| `century` | Century | 100コミット達成 | Silver |
| `thousand_cuts` | Thousand Cuts | 1,000コミット達成 | Gold |
| `legendary` | Legendary | 10,000コミット達成 | Platinum |

##### 🔥 ストリークバッジ

| ID | 名前 | 条件 | レア度 |
|----|------|------|--------|
| `on_fire` | On Fire | 7日連続コミット | Bronze |
| `unstoppable` | Unstoppable | 30日連続コミット | Silver |
| `immortal` | Immortal | 365日連続コミット | Platinum |

##### 🤝 コラボレーションバッジ

| ID | 名前 | 条件 | レア度 |
|----|------|------|--------|
| `team_player` | Team Player | 初レビュー | Bronze |
| `mentor` | Mentor | 50レビュー達成 | Silver |
| `guardian` | Guardian | 100 PRマージ | Gold |

##### 🌟 品質バッジ

| ID | 名前 | 条件 | レア度 |
|----|------|------|--------|
| `clean_coder` | Clean Coder | PRマージ率90%以上（10PR以上） | Gold |
| `bug_hunter` | Bug Hunter | 50 Issue解決 | Silver |
| `polyglot` | Polyglot | 5言語以上使用 | Silver |

##### 🎯 チャレンジバッジ

| ID | 名前 | 条件 | レア度 |
|----|------|------|--------|
| `early_bird` | Early Bird | 週間目標を月曜に達成 | Silver |
| `overachiever` | Overachiever | 週間目標を200%達成 | Gold |
| `consistent` | Consistent | 4週連続で目標達成 | Gold |

#### 3.4.3 レア度ビジュアル

| レア度 | カラー | エフェクト |
|--------|--------|-----------|
| Bronze | #CD7F32 | なし |
| Silver | #C0C0C0 | 光沢 |
| Gold | #FFD700 | ゴールドグロー |
| Platinum | #E5E4E2 | パーティクル |

---

### 3.5 ストリークシステム

#### 3.5.1 ユーザーストーリー

> 開発者として、連続でコミットした日数を見たい。継続のモチベーションを維持するために。

#### 3.5.2 機能詳細

| 機能 | 説明 |
|------|------|
| 現在ストリーク | 現在の連続コミット日数 |
| 最長ストリーク | 過去最長の連続日数 |
| ストリーク継続判定 | 日付変更時に前日のコミット有無を確認 |
| ストリークリセット | 1日でもコミットがなければリセット |

#### 3.5.3 ストリークボーナス

| 連続日数 | ボーナスXP |
|---------|-----------|
| 7日 | +50 XP |
| 14日 | +100 XP |
| 30日 | +200 XP |
| 100日 | +500 XP |
| 365日 | +1,000 XP |

---

### 3.6 チャレンジシステム

#### 3.6.1 ユーザーストーリー

> 開発者として、週間の目標を設定したい。計画的に開発を進め、達成感を得るために。

#### 3.6.2 チャレンジタイプ

| タイプ | 期間 | 生成タイミング |
|--------|------|---------------|
| Daily | 1日 | 毎日0:00 |
| Weekly | 1週間 | 毎週月曜0:00 |

#### 3.6.3 自動生成ロジック

```rust
/// 週次チャレンジの自動生成
/// 過去4週間の平均値 × 1.1 を目標に設定
fn generate_weekly_challenge(
    metric_type: MetricType,
    past_4_weeks: &[u32; 4],
) -> Challenge {
    let average = past_4_weeks.iter().sum::<u32>() / 4;
    let target = (average as f32 * 1.1).ceil() as u32;
    
    Challenge {
        challenge_type: ChallengeType::Weekly,
        target_metric: metric_type,
        target_value: target.max(1), // 最低1
        reward_xp: calculate_reward_xp(metric_type, target),
        // ...
    }
}
```

#### 3.6.4 報酬XP計算

| メトリクス | 目標1あたりのXP |
|-----------|----------------|
| Commits | 10 XP |
| PullRequests | 40 XP |
| Reviews | 20 XP |
| Issues | 25 XP |

---

### 3.7 オフライン対応

#### 3.7.1 ユーザーストーリー

> 開発者として、オフラインでもアプリを使いたい。ネットワーク環境に依存せず、データを確認するために。

#### 3.7.2 キャッシュ戦略

| データタイプ | キャッシュ有効期限 | 優先度 |
|-------------|------------------|--------|
| Contribution Graph | 1時間 | 高 |
| User Stats | 15分 | 高 |
| Badges | 24時間 | 中 |
| Challenge Progress | 5分 | 高 |

#### 3.7.3 Stale-While-Revalidate パターン

```
1. リクエスト時にキャッシュを即座に返す（Stale）
2. バックグラウンドで最新データを取得（Revalidate）
3. 新しいデータでキャッシュを更新
4. UIを更新
```

---

## 4. データベース設計

### 4.1 ER図

```
┌─────────────┐     ┌─────────────────┐     ┌─────────────┐
│   users     │     │   user_stats    │     │   badges    │
├─────────────┤     ├─────────────────┤     ├─────────────┤
│ id (PK)     │────<│ user_id (FK)    │     │ id (PK)     │
│ github_id   │     │ total_xp        │     │ user_id (FK)│>────┐
│ username    │     │ current_level   │     │ badge_type  │     │
│ avatar_url  │     │ current_streak  │     │ badge_id    │     │
│ access_token│     │ longest_streak  │     │ earned_at   │     │
│ created_at  │     │ updated_at      │     └─────────────┘     │
└─────────────┘     └─────────────────┘                         │
       │                                                        │
       │            ┌─────────────────┐     ┌─────────────┐     │
       │            │ activity_cache  │     │ challenges  │     │
       │            ├─────────────────┤     ├─────────────┤     │
       └───────────<│ user_id (FK)    │     │ id (PK)     │     │
                    │ data_type       │     │ user_id (FK)│>────┘
                    │ data_json       │     │ type        │
                    │ fetched_at      │     │ target      │
                    └─────────────────┘     │ current     │
                                            │ status      │
                                            └─────────────┘
```

### 4.2 テーブル定義

```sql
-- ユーザー情報
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    github_id INTEGER UNIQUE NOT NULL,
    username TEXT NOT NULL,
    avatar_url TEXT,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    token_expires_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ユーザー統計
CREATE TABLE user_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER UNIQUE NOT NULL,
    total_xp INTEGER DEFAULT 0,
    current_level INTEGER DEFAULT 1,
    current_streak INTEGER DEFAULT 0,
    longest_streak INTEGER DEFAULT 0,
    last_activity_date DATE,
    total_commits INTEGER DEFAULT 0,
    total_prs INTEGER DEFAULT 0,
    total_reviews INTEGER DEFAULT 0,
    total_issues INTEGER DEFAULT 0,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- バッジ取得履歴
CREATE TABLE badges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    badge_type TEXT NOT NULL,
    badge_id TEXT NOT NULL,
    earned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, badge_id)
);

-- チャレンジ
CREATE TABLE challenges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    challenge_type TEXT NOT NULL, -- 'daily' | 'weekly'
    target_metric TEXT NOT NULL,  -- 'commits' | 'prs' | 'reviews' | 'issues'
    target_value INTEGER NOT NULL,
    current_value INTEGER DEFAULT 0,
    reward_xp INTEGER NOT NULL,
    start_date DATETIME NOT NULL,
    end_date DATETIME NOT NULL,
    status TEXT DEFAULT 'active', -- 'active' | 'completed' | 'failed'
    completed_at DATETIME,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- XP履歴
CREATE TABLE xp_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    action_type TEXT NOT NULL,
    xp_amount INTEGER NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- APIキャッシュ
CREATE TABLE activity_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    data_type TEXT NOT NULL,
    data_json TEXT NOT NULL,
    fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, data_type)
);

-- インデックス
CREATE INDEX idx_badges_user_id ON badges(user_id);
CREATE INDEX idx_challenges_user_id ON challenges(user_id);
CREATE INDEX idx_challenges_status ON challenges(status);
CREATE INDEX idx_xp_history_user_id ON xp_history(user_id);
CREATE INDEX idx_activity_cache_expires ON activity_cache(expires_at);
```

---

## 5. UI/UX設計

### 5.1 デザインシステム

#### 5.1.1 カラーパレット（ダークモード・ゲーミング風）

| 用途 | カラー | HEX |
|------|--------|-----|
| Background Primary | Deep Black | `#0D0D0D` |
| Background Secondary | Dark Gray | `#1A1A2E` |
| Background Card | Dark Purple | `#16213E` |
| Accent Primary | Neon Cyan | `#00F5FF` |
| Accent Secondary | Neon Purple | `#BF00FF` |
| Accent Tertiary | Neon Pink | `#FF00F5` |
| Success | Neon Green | `#00FF85` |
| Warning | Neon Orange | `#FF9500` |
| Error | Neon Red | `#FF0055` |
| Text Primary | White | `#FFFFFF` |
| Text Secondary | Light Gray | `#A0A0A0` |

#### 5.1.2 タイポグラフィ

| 要素 | フォント | サイズ | ウェイト |
|------|---------|--------|---------|
| Heading 1 | Orbitron | 32px | Bold |
| Heading 2 | Orbitron | 24px | Bold |
| Heading 3 | Rajdhani | 20px | SemiBold |
| Body | Rajdhani | 16px | Regular |
| Caption | Rajdhani | 14px | Light |
| Number/Stats | Share Tech Mono | 24px | Regular |

#### 5.1.3 エフェクト

| エフェクト | 適用箇所 |
|-----------|---------|
| Neon Glow | アクセント要素、ボタン |
| Glassmorphism | カード背景 |
| Gradient Border | 重要なカード |
| Particle Animation | レベルアップ、バッジ取得 |

### 5.2 画面レイアウト

#### 5.2.1 ホーム画面（Desktop）

```
┌─────────────────────────────────────────────────────────────────────────┐
│ [Sidebar - 60px]  │                 HOME DASHBOARD                      │
│                   │                                                      │
│ ┌───────────────┐ │  ┌──────────────────────────────────────────────┐   │
│ │ 🏠            │ │  │ PROFILE & LEVEL                              │   │
│ │ Home          │ │  │ ┌──────┐                                     │   │
│ └───────────────┘ │  │ │Avatar│  @username                          │   │
│ ┌───────────────┐ │  │ └──────┘  Level 23                           │   │
│ │ 🔧            │ │  │                                              │   │
│ │ Tools         │ │  │  ████████████████░░░░░░░  2,340 / 3,000 XP   │   │
│ └───────────────┘ │  │                                              │   │
│ ┌───────────────┐ │  │  🔥 15 days   ⭐ 1,234   🏅 12              │   │
│ │ ⚙️            │ │  │     streak       commits     badges          │   │
│ │ Settings      │ │  └──────────────────────────────────────────────┘   │
│ └───────────────┘ │                                                      │
│                   │  ┌─────────────────────┐ ┌────────────────────────┐ │
│                   │  │ WEEKLY ACTIVITY     │ │ CHALLENGES             │ │
│                   │  │                     │ │                        │ │
│                   │  │  M  T  W  T  F  S  S │ │ ☑ 5 commits    4/5   │ │
│                   │  │  █  █  █  ░  ░  ░  ░ │ │   ██████████░░ +50XP │ │
│                   │  │  5  3  8  -  -  -  - │ │                        │ │
│                   │  │                     │ │ ☐ 2 PRs        1/2   │ │
│                   │  │  Total: 16 commits  │ │   █████░░░░░░░ +80XP │ │
│                   │  └─────────────────────┘ └────────────────────────┘ │
│                   │                                                      │
│                   │  ┌──────────────────────────────────────────────┐   │
│                   │  │ RECENT ACHIEVEMENTS                          │   │
│                   │  │                                              │   │
│                   │  │  🏅 Century    🔥 On Fire    🤝 Team Player   │   │
│                   │  │  100 commits   7 day streak   First review   │   │
│                   │  └──────────────────────────────────────────────┘   │
│                   │                                                      │
│                   │  ┌──────────────────────────────────────────────┐   │
│                   │  │ CONTRIBUTION GRAPH                           │   │
│                   │  │                                              │   │
│                   │  │  [GitHub-style yearly contribution graph]    │   │
│                   │  │                                              │   │
│                   │  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.3 コンポーネント一覧

| コンポーネント | 説明 | 優先度 |
|---------------|------|--------|
| `ProfileCard` | ユーザー情報・レベル表示 | P0 |
| `XpProgressBar` | 経験値プログレスバー | P0 |
| `StatsDisplay` | ストリーク・コミット数・バッジ数 | P0 |
| `WeeklyActivity` | 週間活動グラフ | P1 |
| `ChallengeCard` | チャレンジ進捗表示 | P1 |
| `BadgeGrid` | バッジ一覧表示 | P1 |
| `ContributionGraph` | GitHub草グラフ | P1 |
| `LevelUpModal` | レベルアップ演出 | P2 |
| `BadgeUnlockModal` | バッジ取得演出 | P2 |

### 5.4 アニメーション

| アニメーション | トリガー | 演出 |
|---------------|---------|------|
| XP Gain | XP獲得時 | +XP が浮かび上がって消える |
| Level Up | レベルアップ時 | パーティクル爆発 + モーダル |
| Badge Unlock | バッジ取得時 | シャイン + モーダル |
| Streak Continue | ストリーク継続 | 火のアニメーション強化 |
| Progress Update | 進捗更新時 | スムーズなバー伸長 |

---

## 6. 開発フェーズ

### 6.1 フェーズ概要

| Phase | 内容 | Issue | 期間目安 |
|-------|------|-------|---------|
| 0 | 基盤整備（OAuth, SQLite, API） | #5 | 2週間 |
| 1 | 基本メトリクス & ホーム画面UI | #6 | 2週間 |
| 2 | レベルシステム | #7 | 1週間 |
| 3 | バッジ・ストリーク | #8 | 2週間 |
| 4 | チャレンジ機能 | #9 | 1週間 |
| 5 | オフライン対応 | #10 | 1週間 |

### 6.2 MVP（Minimum Viable Product）

**Phase 0 + Phase 1 + Phase 2** を MVP とする。

MVP完了時点で以下が動作する：
- GitHub OAuth ログイン
- 基本メトリクス表示
- レベル・XP表示
- ホーム画面の基本レイアウト

---

## 7. テスト戦略

### 7.1 TDD（テスト駆動開発）

各機能は以下の順序で実装：

1. **Red**: テストを先に書く（失敗する）
2. **Green**: 最小限の実装でテストを通す
3. **Refactor**: コードを整理

### 7.2 テストカテゴリ

| カテゴリ | ツール | 対象 |
|---------|--------|------|
| Unit Test | `cargo test` | ビジネスロジック |
| Integration Test | `cargo test` | API連携、DB操作 |
| UI Test | `wasm-bindgen-test` | Leptosコンポーネント |
| E2E Test | Tauri Driver | アプリ全体 |

### 7.3 カバレッジ目標

| 対象 | カバレッジ目標 |
|------|---------------|
| ビジネスロジック | 90%以上 |
| API連携 | 80%以上 |
| UIコンポーネント | 70%以上 |

---

## 8. セキュリティ要件

### 8.1 認証・認可

| 要件 | 対応 |
|------|------|
| トークン保存 | SQLiteで暗号化保存（AES-256） |
| PKCE | OAuth フローでPKCE使用 |
| スコープ最小化 | 必要最小限のスコープのみ要求 |

### 8.2 データ保護

| 要件 | 対応 |
|------|------|
| ローカルデータ | SQLiteファイル暗号化 |
| 通信 | HTTPS必須 |
| キャッシュ | 有効期限後は自動削除 |

---

## 9. 非機能要件

### 9.1 パフォーマンス

| 指標 | 目標 |
|------|------|
| 初回起動時間 | 3秒以内 |
| ホーム画面描画 | 1秒以内 |
| API応答（キャッシュあり） | 100ms以内 |
| API応答（キャッシュなし） | 2秒以内 |

### 9.2 可用性

| 指標 | 目標 |
|------|------|
| オフライン対応 | キャッシュデータで動作 |
| エラーリカバリ | 自動リトライ（3回） |

---

## 10. 将来の拡張

### 10.1 チーム機能（Phase 6以降）

- チームダッシュボード
- チームランキング
- チーム目標設定
- GitHub Organization連携

### 10.2 その他の拡張案

- 他サービス連携（GitLab, Bitbucket）
- カスタムバッジ作成
- テーマカスタマイズ
- 通知機能（デスクトップ通知）
- ウィジェット（メニューバー常駐）

---

## 11. 用語集

| 用語 | 説明 |
|------|------|
| XP | Experience Points（経験値） |
| ストリーク | 連続達成日数 |
| バッジ | アチーブメント（実績） |
| チャレンジ | 期間限定の目標 |
| Contribution Graph | GitHubの草グラフ |
| PKCE | Proof Key for Code Exchange |

---

## 12. 参考資料

- [GitHub OAuth Documentation](https://docs.github.com/en/apps/oauth-apps)
- [GitHub GraphQL API](https://docs.github.com/en/graphql)
- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Leptos Documentation](https://leptos.dev/)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2024-11-26 | - | Initial draft |

