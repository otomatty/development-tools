# アーキテクチャ

Development Tools のシステムアーキテクチャについて説明します。

---

## 📋 目次

- [システム概要](#システム概要)
- [レイヤー構成](#レイヤー構成)
- [フロントエンド](#フロントエンド)
- [バックエンド](#バックエンド)
- [データフロー](#データフロー)
- [データベース](#データベース)
- [外部連携](#外部連携)
- [モジュール詳細](#モジュール詳細)

---

## システム概要

```text
┌─────────────────────────────────────────────────────────────────────────┐
│          Frontend (Solid.js + Leptos/WASM - 段階的移行中)                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐                  │
│  │   HomePage  │  │ ToolDetail  │  │  SettingsPage   │                  │
│  │(Gamification)│ │  (Runner)   │  │                 │                  │
│  └──────┬──────┘  └──────┬──────┘  └────────┬────────┘                  │
│         │                │                   │                           │
│         └────────────────┴───────────────────┘                           │
│                                   │                                      │
│  ┌────────────────────────────────┴──────────────────────────────────┐ │
│  │  UI Components (Solid.js) - Phase 3-1実装済み                      │ │
│  │  Button, Input, Modal, DropdownMenu, Toast                        │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                   │                                      │
│                            tauri_api.rs                                  │
│                      (Tauri IPC Wrapper)                                 │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                              Tauri IPC
                             (invoke/listen)
                                    │
┌──────────────────────────────────────────────────────────────────────────┐
│                        Backend (Tauri + Rust)                            │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                        commands/                                  │   │
│  │  ┌────────┐ ┌───────────────┐ ┌──────────┐ ┌────────┐│   │
│  │  │auth    │ │gamification   │ │github    │ │settings││   │
│  │  └───┬────┘ └───────┬───────┘ └────┬─────┘ └───┬────┘│   │
│  └──────│──────────────│──────────────│───────────│─────┘   │
│         │              │              │           │          │
│  ┌──────▼──────────────▼──────────────▼───────────▼─────┐   │
│  │                         Core Services                            │   │
│  │  ┌─────────────┐  ┌─────────────┐                                │   │
│  │  │   auth/     │  │   github/   │                                │   │
│  │  │ (OAuth)     │  │ (API Client)│                                │   │
│  │  └─────────────┘  └─────────────┘                                │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                    │                                     │
│  ┌─────────────────────────────────▼────────────────────────────────┐   │
│  │                          database/                                │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │   │
│  │  │connection│  │migrations│  │repository│  │     models/      │  │   │
│  │  └──────────┘  └──────────┘  └──────────┘  │ user, badge, xp, │  │   │
│  │                                            │ level, settings  │  │   │
│  │                                            └──────────────────┘  │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                    │                                     │
│                               SQLite                                     │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                              External APIs
                                    │
                    ┌───────────────┴───────────────┐
                    │                               │
              ┌─────▼─────┐                  ┌──────▼──────┐
              │  GitHub   │                  │ CLI Tools   │
              │   API     │                  │ (Subprocess)│
              └───────────┘                  └─────────────┘
```

---

## レイヤー構成

### 3 層アーキテクチャ

| レイヤー           | 技術               | 責務                       |
| ------------------ | ------------------ | -------------------------- |
| **Presentation**   | Leptos/WASM        | UI 表示、ユーザー操作      |
| **Application**    | Tauri Commands     | ビジネスロジック、IPC 処理 |
| **Infrastructure** | SQLite, GitHub API | データ永続化、外部連携     |

---

## フロントエンド

### 技術スタック

- **Solid.js**: TypeScript ベースのリアクティブフレームワーク（Phase 2-3で移行）
- **Leptos 0.7**: Rust ベースのリアクティブフレームワーク（段階的に移行中）
- **WASM**: WebAssembly にコンパイル（Leptosコンポーネント用）
- **Tailwind CSS**: ユーティリティファースト CSS

### ディレクトリ構造

```
src/
├── App.tsx                   # メインアプリケーションコンポーネント（Solid.js）
├── main.tsx                  # エントリーポイント（Solid.js）
├── app.rs                    # メインアプリケーションコンポーネント（Leptos、段階的に移行中）
├── main.rs                   # エントリーポイント（Leptos）
├── tauri_api.rs              # Tauri IPC呼び出しラッパー
├── pages/                    # ページコンポーネント（Solid.js）
│   ├── Home/
│   ├── Projects/
│   ├── Settings/
│   └── ...
├── components/
│   ├── mod.rs                # コンポーネント公開（Leptos）
│   ├── icons/                # アイコンコンポーネント（Solid.js版実装済み）
│   │   ├── Icon.tsx          # lucide-solidを使用したアイコンコンポーネント
│   │   └── index.ts
│   ├── layouts/              # レイアウトコンポーネント（Solid.js版実装済み）
│   │   ├── Sidebar/          # サイドバーナビゲーション
│   │   │   ├── Sidebar.tsx
│   │   │   ├── SidebarItem.tsx
│   │   │   ├── Sidebar.spec.md
│   │   │   └── index.ts
│   │   ├── MainLayout/       # メインレイアウト
│   │   │   ├── MainLayout.tsx
│   │   │   ├── MainLayout.spec.md
│   │   │   └── index.ts
│   │   ├── OfflineBanner/    # オフラインバナー
│   │   │   ├── OfflineBanner.tsx
│   │   │   ├── OfflineBanner.spec.md
│   │   │   └── index.ts
│   │   └── index.ts
│   ├── ui/                   # UIコンポーネント（Leptos + Solid.js）
│   │   ├── button/           # Button, IconButton（Solid.js版実装済み）
│   │   │   ├── Button.tsx
│   │   │   ├── Button.spec.md
│   │   │   ├── button.rs     # Leptos版（段階的に削除予定）
│   │   │   └── index.ts
│   │   ├── form/             # Input, TextArea, LabeledInput（Solid.js版実装済み）
│   │   │   ├── Input.tsx
│   │   │   ├── Input.spec.md
│   │   │   ├── input.rs      # Leptos版（段階的に削除予定）
│   │   │   └── index.ts
│   │   ├── dialog/           # Modal関連（Solid.js版実装済み）
│   │   │   ├── Modal.tsx
│   │   │   ├── Modal.spec.md
│   │   │   ├── modal.rs      # Leptos版（段階的に削除予定）
│   │   │   └── index.ts
│   │   ├── dropdown/         # DropdownMenu（Solid.js版実装済み）
│   │   │   ├── DropdownMenu.tsx
│   │   │   ├── DropdownMenu.spec.md
│   │   │   ├── dropdown_menu.rs # Leptos版（段階的に削除予定）
│   │   │   └── index.ts
│   │   └── feedback/         # Toast（Solid.js版実装済み）
│   │       ├── Toast.tsx
│   │       ├── Toast.spec.md
│   │       ├── toast.rs      # Leptos版（段階的に削除予定）
│   │       └── index.ts
│   ├── animation_context.rs  # アニメーション状態管理
│   ├── confirm_dialog.rs     # 確認ダイアログ
│   ├── icons.rs              # SVGアイコン（Leptos版、段階的に削除予定）
│   ├── sidebar.rs            # サイドバーナビゲーション（Leptos版、段階的に削除予定）
│   ├── home/                 # ホーム画面（ゲーミフィケーション）
│   │   ├── badge_grid.rs     # バッジ一覧
│   │   ├── challenge_card.rs # チャレンジカード
│   │   ├── contribution_graph.rs # コントリビューショングラフ
│   │   ├── login_card.rs     # ログインUI
│   │   ├── profile_card.rs   # プロフィールカード
│   │   ├── stats_display.rs  # 統計表示
│   │   └── xp_notification.rs# XP通知
│   ├── settings/             # 設定ページ
│   └── skeleton/             # ローディングスケルトン
├── stores/                   # Solid.jsストア（状態管理）
│   ├── animationStore.ts
│   ├── authStore.ts
│   ├── navigationStore.ts
│   └── ...
├── hooks/                    # Solid.jsフック
│   ├── useToast.ts           # Toast用フック（新規作成）
│   └── ...
└── types/
    ├── index.ts              # 型定義公開（TypeScript）
    ├── ui.ts                 # UIコンポーネント用型定義（新規作成）
    ├── auth.ts               # 認証関連型
    ├── challenge.ts          # チャレンジ関連型
    ├── gamification.ts       # ゲーミフィケーション関連型
    └── settings.ts           # 設定関連型
```

### 状態管理

**Solid.js（新規）**:

```typescript
// リアクティブな状態
const [count, setCount] = createSignal(0);

// ストアによるグローバル状態
import { useAnimation } from './stores/animationStore';
const animation = useAnimation();
```

**Leptos（段階的に移行中）**:

```rust
// リアクティブな状態
let (count, set_count) = signal(0);

// コンテキストによるグローバル状態
provide_context(AnimationContext::new(true));
let ctx = use_context::<AnimationContext>();
```

### ページ遷移

**Solid.js（Phase 2-3で実装済み）**:

`@solidjs/router` を使用したルーティング：

```typescript
import { Router, Routes, Route } from '@solidjs/router';

<Router>
  <Routes>
    <Route path="/" component={Home} />
    <Route path="/projects" component={Projects} />
    <Route path="/settings" component={Settings} />
  </Routes>
</Router>
```

**Leptos（段階的に移行中）**:

`AppPage` enum によるシンプルなルーティング：

```rust
pub enum AppPage {
    Home,                // ゲーミフィケーションダッシュボード
    Projects,            // プロジェクト一覧
    ProjectDetail(i64),  // プロジェクト詳細
    Settings,            // 設定
    XpHistory,           // XP履歴
}
```

### レイアウトコンポーネント（Phase 3-2で実装済み）

レイアウトコンポーネントはSolid.js版が実装済み：

- **MainLayout**: アプリ全体のレイアウト（Sidebar + メインコンテンツエリア）
- **Sidebar**: メインナビゲーション（@solidjs/routerと統合）
- **SidebarItem**: ナビゲーション項目（アクティブ状態のハイライト対応）
- **OfflineBanner**: オフライン時の警告バナー（ネットワーク状態表示）
- **Icon**: lucide-solidを使用したアイコンコンポーネント（既存アイコン名との互換性を保持）

### UIコンポーネント（Phase 3-1で実装済み）

基本UIコンポーネントはSolid.js版が実装済み：

- **Button / IconButton**: 6バリアント、3サイズ、isLoading対応
- **Input / TextArea / LabeledInput**: 6種類のinputType、3サイズ対応
- **Modal / ModalHeader / ModalBody / ModalFooter**: Portal対応、ESCキー、オーバーレイクリック対応
- **DropdownMenu / DropdownMenuItem / DropdownMenuDivider**: Context API使用、ESCキー対応
- **Toast / InlineToast**: 4タイプ対応、自動非表示対応

使用方法：

```typescript
import { Button } from './components/ui/button';
import { Input, LabeledInput } from './components/ui/form';
import { Modal, ModalHeader, ModalBody, ModalFooter } from './components/ui/dialog';
import { DropdownMenu, DropdownMenuItem } from './components/ui/dropdown';
import { Toast, useToast } from './components/ui/feedback';

// Button使用例
<Button variant="primary" size="md" onClick={handleClick}>
  Click Me
</Button>

// Input使用例
const [value, setValue] = createSignal('');
<LabeledInput
  value={value}
  onInput={setValue}
  label="Username"
  required
/>

// Modal使用例
const [isOpen, setIsOpen] = createSignal(false);
<Modal visible={isOpen} onClose={() => setIsOpen(false)}>
  <ModalHeader onClose={() => setIsOpen(false)}>Title</ModalHeader>
  <ModalBody>Content</ModalBody>
  <ModalFooter>Actions</ModalFooter>
</Modal>

// Toast使用例
const toast = useToast();
toast.success("保存しました");
```

---

## バックエンド

### 技術スタック

- **Tauri 2.0**: デスクトップアプリケーションフレームワーク
- **SQLite (sqlx)**: 組み込みデータベース
- **reqwest**: HTTP クライアント

### ディレクトリ構造

```
src-tauri/src/
├── lib.rs                    # ライブラリエントリーポイント
├── main.rs                   # アプリケーションエントリーポイント
├── types.rs                  # 共通型定義
├── auth/                     # 認証モジュール
│   ├── mod.rs
│   ├── crypto.rs             # トークン暗号化
│   ├── oauth.rs              # OAuth Device Flow
│   └── token.rs              # トークン管理
├── commands/                 # Tauriコマンド（IPC）
│   ├── mod.rs
│   ├── auth.rs               # 認証コマンド
│   ├── challenge.rs          # チャレンジコマンド
│   ├── gamification.rs       # ゲーミフィケーションコマンド
│   ├── github.rs             # GitHub関連コマンド
│   └── settings.rs           # 設定コマンド
├── database/                 # データベースモジュール
│   ├── mod.rs
│   ├── connection.rs         # 接続管理
│   ├── migrations.rs         # マイグレーション
│   ├── challenge.rs          # チャレンジロジック
│   ├── models/               # データモデル
│   │   ├── badge.rs          # バッジ
│   │   ├── cache.rs          # キャッシュ
│   │   ├── challenge.rs      # チャレンジ
│   │   ├── code_stats.rs     # コード統計
│   │   ├── level.rs          # レベル
│   │   ├── settings.rs       # 設定
│   │   ├── streak.rs         # ストリーク
│   │   ├── user.rs           # ユーザー
│   │   └── xp.rs             # XP
│   └── repository/           # リポジトリパターン
├── github/                   # GitHub API クライアント
│   ├── mod.rs
│   ├── client.rs             # HTTPクライアント
│   └── types.rs              # API型定義
└── utils/                    # ユーティリティ
```

### Tauri コマンド

フロントエンドから呼び出せる IPC 関数：

```rust
#[tauri::command]
pub async fn get_github_stats(
    state: State<'_, AppState>,
) -> Result<GitHubStats, String> {
    // ...
}
```

フロントエンドからの呼び出し：

```rust
// tauri_api.rs
pub async fn get_github_stats() -> Result<GitHubStats, String> {
    invoke("get_github_stats", JsValue::NULL).await
}
```

---

## データフロー

### 1. GitHub 統計同期フロー

```
User Click "Sync" → tauri_api::sync_github_stats()
                               │
                        commands::sync_github_stats()
                               │
            ┌──────────────────┴──────────────────┐
            │                                      │
    github::client                          database::
    GET /graphql                            save_github_stats()
            │                                      │
            └──────────────────┬──────────────────┘
                               │
                        Calculate XP diff
                               │
                        add_xp() if needed
                               │
                        Check level up
                               │
                        Emit "level-up" event
                               │
                        Frontend shows notification
```

### 2. 認証フロー（Device Flow）

```
User Click "Login" → start_device_flow()
                               │
                        POST /login/device/code
                               │
                        Return user_code, device_code
                               │
                        Show user_code to user
                        Open browser to verification URL
                               │
                        poll_device_token() (every 5 seconds)
                               │
                        POST /login/oauth/access_token
                               │
                        Access Token received
                               │
                        Encrypt & store token
                               │
                        Fetch user info
                               │
                        Create/update user in DB
```

---

## データベース

### テーブル一覧

| テーブル                 | 説明                 |
| ------------------------ | -------------------- |
| `users`                  | ユーザー情報         |
| `github_stats`           | GitHub 統計データ    |
| `github_stats_snapshots` | 日次スナップショット |
| `xp_history`             | XP 履歴              |
| `badges`                 | 獲得バッジ           |
| `badge_definitions`      | バッジ定義           |
| `streaks`                | ストリーク情報       |
| `challenges`             | チャレンジ           |
| `settings`               | ユーザー設定         |
| `cache`                  | キャッシュデータ     |

### ER 図（簡略版）

```
┌─────────────┐       ┌─────────────────┐
│   users     │       │  github_stats   │
├─────────────┤       ├─────────────────┤
│ id (PK)     │◄──────│ user_id (FK)    │
│ github_id   │       │ total_commits   │
│ username    │       │ total_prs       │
│ display_name│       │ ...             │
│ avatar_url  │       └─────────────────┘
│ xp          │
│ level       │       ┌─────────────────┐
│ ...         │       │   xp_history    │
└─────────────┘       ├─────────────────┤
      │               │ user_id (FK)    │◄─┐
      │               │ amount          │  │
      │               │ source          │  │
      │               │ ...             │  │
      │               └─────────────────┘  │
      │                                    │
      │               ┌─────────────────┐  │
      │               │     badges      │  │
      └──────────────►├─────────────────┤  │
                      │ user_id (FK)    │──┘
                      │ badge_id        │
                      │ ...             │
                      └─────────────────┘
```

---

## 外部連携

### GitHub API

- **認証**: Device Flow（OAuth 2.0）
- **エンドポイント**: GraphQL API v4
- **取得データ**: ユーザー情報、コントリビューション、リポジトリ統計

---

## モジュール詳細

### commands/ - Tauri コマンドモジュール

| ファイル          | 責務                            |
| ----------------- | ------------------------------- |
| `auth.rs`         | 認証（Device Flow、ログアウト） |
| `challenge.rs`    | チャレンジの CRUD               |
| `gamification.rs` | XP、レベル、バッジ操作          |
| `github.rs`       | GitHub 統計の取得・同期         |
| `settings.rs`     | 設定の取得・更新                |

### database/models/ - データモデル

| ファイル       | 責務                       |
| -------------- | -------------------------- |
| `user.rs`      | ユーザーの作成・更新・取得 |
| `badge.rs`     | バッジの付与・確認         |
| `xp.rs`        | XP の追加・履歴取得        |
| `level.rs`     | レベル計算・取得           |
| `streak.rs`    | ストリーク計算             |
| `settings.rs`  | 設定の保存・読み込み       |
| `challenge.rs` | チャレンジの管理           |
| `cache.rs`     | キャッシュ管理             |

### auth/ - 認証モジュール

| ファイル    | 責務                 |
| ----------- | -------------------- |
| `oauth.rs`  | Device Flow 実装     |
| `token.rs`  | トークンの保存・取得 |
| `crypto.rs` | AES-GCM 暗号化       |

---

## セキュリティ考慮事項

### トークン管理

- アクセストークンは AES-GCM で暗号化して SQLite に保存
- 暗号化キーは OS のキーチェーンに保存（将来実装予定）

### API 通信

- すべての API 通信は HTTPS
- レート制限の遵守（GitHub API: 5000 requests/hour）

### ファイルアクセス

- CLI ツールの実行はユーザーが指定したディレクトリのみ

---

## パフォーマンス考慮事項

### フロントエンド

- コンポーネントの遅延読み込み
- 不要な再レンダリングの防止（`Memo`、`Effect`の活用）

### バックエンド

- データベース接続のプール化
- GitHub API レスポンスのキャッシュ
- 非同期処理の活用（Tokio）

### データベース

- インデックスの適切な設定
- WAL モードによる並行アクセス対応
