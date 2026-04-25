# Tauri Commands API 仕様書

フロントエンドから呼び出せる Tauri コマンド（IPC）の一覧と仕様です。

---

## 📋 目次

- [認証コマンド](#認証コマンド)
- [GitHub コマンド](#githubコマンド)
- [ゲーミフィケーションコマンド](#ゲーミフィケーションコマンド)
- [チャレンジコマンド](#チャレンジコマンド)
- [設定コマンド](#設定コマンド)
- [モックサーバーコマンド](#モックサーバーコマンド)

---

## 認証コマンド

GitHub OAuth 認証（Device Flow）に関するコマンド。

### `start_device_flow`

Device Flow を開始します。

**パラメータ**: なし

**戻り値**: `DeviceFlowResponse`

```typescript
interface DeviceFlowResponse {
  deviceCode: string;
  userCode: string;
  verificationUri: string;
  expiresIn: number;
  interval: number;
}
```

---

### `poll_device_token`

トークンの取得をポーリングします。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `device_code` | `string` | デバイスコード |

**戻り値**: `DeviceFlowPollResult`

```typescript
type DeviceFlowPollResult =
  | { status: "pending" }
  | { status: "success"; user: GitHubUser }
  | { status: "error"; message: string };
```

---

### `cancel_device_flow`

Device Flow をキャンセルします。

**パラメータ**: なし

**戻り値**: `()`

---

### `get_auth_state`

現在の認証状態を取得します。

**パラメータ**: なし

**戻り値**: `AuthState`

```typescript
interface AuthState {
  isAuthenticated: boolean;
  user?: GitHubUser;
}
```

---

### `logout`

ログアウトします。

**パラメータ**: なし

**戻り値**: `()`

---

### `get_current_user`

現在のユーザー情報を取得します。

**パラメータ**: なし

**戻り値**: `Option<GitHubUser>`

```typescript
interface GitHubUser {
  id: number;
  login: string;
  name?: string;
  avatarUrl: string;
  email?: string;
}
```

---

### `validate_token`

トークンの有効性を確認します。

**パラメータ**: なし

**戻り値**: `bool`

---

### `open_url`

ブラウザで URL を開きます。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `url` | `string` | 開く URL |

**戻り値**: `()`

---

## GitHub コマンド

GitHub API との連携に関するコマンド。

### `get_github_user`

GitHub ユーザー情報を取得します。

**パラメータ**: なし

**戻り値**: `GitHubUser`

---

### `get_github_stats`

GitHub 統計を取得します（キャッシュから）。

**パラメータ**: なし

**戻り値**: `GitHubStats`

```typescript
interface GitHubStats {
  totalCommits: number;
  totalPrs: number;
  totalReviews: number;
  totalIssues: number;
  totalStarsReceived: number;
  totalContributions: number;
  updatedAt: string;
}
```

---

### `get_user_stats`

ユーザー統計（XP、レベル、ストリーク）を取得します。

**パラメータ**: なし

**戻り値**: `UserStats`

```typescript
interface UserStats {
  totalXp: number;
  currentLevel: number;
  currentStreak: number;
  longestStreak: number;
  lastActivityDate?: string;
  totalCommits: number;
  totalPrs: number;
  totalReviews: number;
  totalIssues: number;
}
```

---

### `sync_github_stats`

GitHub 統計を同期（API 呼び出し）します。

**パラメータ**: なし

**戻り値**: `GitHubStats`

**副作用**: XP が付与される場合があります

---

### `get_contribution_calendar`

コントリビューションカレンダーデータを取得します。

**パラメータ**: なし

**戻り値**: `ContributionCalendar`

```typescript
interface ContributionCalendar {
  totalContributions: number;
  weeks: ContributionWeek[];
}

interface ContributionWeek {
  contributionDays: ContributionDay[];
}

interface ContributionDay {
  date: string;
  contributionCount: number;
  contributionLevel: string; // "NONE" | "FIRST_QUARTILE" | "SECOND_QUARTILE" | "THIRD_QUARTILE" | "FOURTH_QUARTILE"
}
```

---

### `get_badges_with_progress`

進捗付きバッジ情報を取得します。

**パラメータ**: なし

**戻り値**: `Vec<BadgeWithProgress>`

```typescript
interface BadgeWithProgress {
  definition: BadgeDefinition;
  earned: boolean;
  earnedAt?: string;
  progress: number;
  progressText: string;
}
```

---

### `get_near_completion_badges`

完了間近のバッジを取得します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `limit` | `number` | 取得件数 |

**戻り値**: `Vec<BadgeWithProgress>`

---

### `sync_code_stats`

コード統計（additions/deletions）を同期します。

**パラメータ**: なし

**戻り値**: `CodeStatsSummary`

---

### `get_code_stats_summary`

コード統計のサマリーを取得します。

**パラメータ**: なし

**戻り値**: `CodeStatsSummary`

```typescript
interface CodeStatsSummary {
  totalAdditions: number;
  totalDeletions: number;
  totalCommits: number;
  lastSyncAt?: string;
}
```

---

### `get_rate_limit_info`

GitHub API のレート制限情報を取得します。

**パラメータ**: なし

**戻り値**: `RateLimitInfo`

```typescript
interface RateLimitInfo {
  remaining: number;
  resetAt: string;
}
```

---

## ゲーミフィケーションコマンド

レベル、XP、バッジに関するコマンド。

### `get_level_info`

レベル情報を取得します。

**パラメータ**: なし

**戻り値**: `LevelInfo`

```typescript
interface LevelInfo {
  currentLevel: number;
  currentXp: number;
  xpForCurrentLevel: number;
  xpForNextLevel: number;
  progress: number; // 0.0 - 1.0
}
```

---

### `add_xp`

XP を追加します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `amount` | `number` | XP 量 |
| `source` | `string` | XP 獲得元 |

**戻り値**: `LevelUpResult`

```typescript
interface LevelUpResult {
  newXp: number;
  newLevel: number;
  leveledUp: boolean;
  previousLevel?: number;
}
```

**イベント**: `level-up`（レベルアップ時）

---

### `get_badges`

獲得済みバッジを取得します。

**パラメータ**: なし

**戻り値**: `Vec<Badge>`

```typescript
interface Badge {
  id: string;
  badgeType: string;
  earnedAt: string;
}
```

---

### `award_badge`

バッジを付与します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `badge_id` | `string` | バッジ ID |

**戻り値**: `bool` - 新規付与されたかどうか

---

### `get_xp_history`

XP 履歴を取得します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `limit` | `number` | 取得件数 |

**戻り値**: `Vec<XpHistoryEntry>`

```typescript
interface XpHistoryEntry {
  actionType: string;
  xpAmount: number;
  description?: string;
  createdAt: string;
}
```

---

### `get_badge_definitions`

バッジ定義一覧を取得します。

**パラメータ**: なし

**戻り値**: `Vec<BadgeDefinition>`

```typescript
interface BadgeDefinition {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  requirement: BadgeRequirement;
}
```

---

## チャレンジコマンド

デイリー/ウィークリーチャレンジに関するコマンド。

### `get_active_challenges`

アクティブなチャレンジを取得します。

**パラメータ**: なし

**戻り値**: `Vec<Challenge>`

```typescript
interface Challenge {
  id: number;
  challengeType: "daily" | "weekly";
  targetMetric: string;
  targetValue: number;
  currentValue: number;
  rewardXp: number;
  startDate: string;
  endDate: string;
  status: "active" | "completed" | "expired";
}
```

---

### `get_all_challenges`

全チャレンジを取得します。

**パラメータ**: なし

**戻り値**: `Vec<Challenge>`

---

### `get_challenges_by_type`

タイプ別チャレンジを取得します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `challenge_type` | `string` | "daily" or "weekly" |

**戻り値**: `Vec<Challenge>`

---

### `create_challenge`

チャレンジを作成します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `challenge` | `CreateChallenge` | チャレンジ情報 |

**戻り値**: `Challenge`

---

### `delete_challenge`

チャレンジを削除します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `challenge_id` | `number` | チャレンジ ID |

**戻り値**: `()`

---

### `update_challenge_progress`

チャレンジの進捗を更新します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `challenge_id` | `number` | チャレンジ ID |
| `new_value` | `number` | 新しい値 |

**戻り値**: `Challenge`

---

### `get_challenge_stats`

チャレンジ統計を取得します。

**パラメータ**: なし

**戻り値**: `ChallengeStats`

```typescript
interface ChallengeStats {
  totalCompleted: number;
  totalExpired: number;
  currentStreak: number;
}
```

---

## 設定コマンド

アプリケーション設定に関するコマンド。

### `get_settings`

設定を取得します。

**パラメータ**: なし

**戻り値**: `UserSettings`

```typescript
interface UserSettings {
  notificationMethod: "app_only" | "os_only" | "both" | "none";
  notifyXpGain: boolean;
  notifyLevelUp: boolean;
  notifyBadgeEarned: boolean;
  notifyStreakUpdate: boolean;
  notifyStreakMilestone: boolean;
  syncIntervalMinutes: number;
  backgroundSync: boolean;
  syncOnStartup: boolean;
  animationsEnabled: boolean;
}
```

---

### `update_settings`

設定を更新します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `settings` | `UserSettings` | 新しい設定 |

**戻り値**: `UserSettings`

---

### `reset_settings`

設定をリセットします。

**パラメータ**: なし

**戻り値**: `UserSettings`

---

### `clear_cache`

キャッシュをクリアします。

**パラメータ**: なし

**戻り値**: `ClearCacheResult`

```typescript
interface ClearCacheResult {
  clearedEntries: number;
}
```

---

### `get_database_info`

データベース情報を取得します。

**パラメータ**: なし

**戻り値**: `DatabaseInfo`

```typescript
interface DatabaseInfo {
  path: string;
  sizeBytes: number;
  sizeFormatted: string;
}
```

---

### `reset_all_data`

全データをリセットします。

**パラメータ**: なし

**戻り値**: `()`

---

### `export_data`

データをエクスポートします。

**パラメータ**: なし

**戻り値**: `ExportData`

---

### `get_sync_intervals`

同期間隔の選択肢を取得します。

**パラメータ**: なし

**戻り値**: `Vec<SyncIntervalOption>`

```typescript
interface SyncIntervalOption {
  value: number;
  label: string;
}
```

---

### `get_app_info`

アプリケーション情報を取得します。

**パラメータ**: なし

**戻り値**: `AppInfo`

```typescript
interface AppInfo {
  version: string;
  tauriVersion: string;
  rustVersion: string;
  buildDate: string;
}
```

---

### `open_external_url`

外部 URL を開きます。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `url` | `string` | URL |

**戻り値**: `()`

---

## モックサーバーコマンド

静的ファイル配信サーバーに関するコマンド。

### `get_mock_server_state`

サーバー状態を取得します。

**パラメータ**: なし

**戻り値**: `MockServerState`

```typescript
interface MockServerState {
  isRunning: boolean;
  port?: number;
  startedAt?: string;
}
```

---

### `start_mock_server`

サーバーを起動します。

**パラメータ**: なし

**戻り値**: `MockServerState`

---

### `stop_mock_server`

サーバーを停止します。

**パラメータ**: なし

**戻り値**: `MockServerState`

---

### `get_mock_server_config`

サーバー設定を取得します。

**パラメータ**: なし

**戻り値**: `MockServerConfig`

```typescript
interface MockServerConfig {
  port: number;
  corsMode: "simple" | "advanced";
  corsOrigins?: string[];
  corsMethods?: string[];
  corsHeaders?: string[];
  corsMaxAge: number;
  showDirectoryListing: boolean;
}
```

---

### `update_mock_server_config`

サーバー設定を更新します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `config` | `MockServerConfig` | 新しい設定 |

**戻り値**: `MockServerConfig`

---

### `get_mock_server_mappings`

ディレクトリマッピング一覧を取得します。

**パラメータ**: なし

**戻り値**: `Vec<MockServerMapping>`

```typescript
interface MockServerMapping {
  id: number;
  virtualPath: string;
  localPath: string;
  enabled: boolean;
}
```

---

### `create_mock_server_mapping`

マッピングを作成します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `mapping` | `CreateMapping` | マッピング情報 |

**戻り値**: `MockServerMapping`

---

### `update_mock_server_mapping`

マッピングを更新します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `mapping` | `MockServerMapping` | 更新情報 |

**戻り値**: `MockServerMapping`

---

### `delete_mock_server_mapping`

マッピングを削除します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `id` | `number` | マッピング ID |

**戻り値**: `()`

---

### `list_mock_server_directory`

ディレクトリ内のファイル一覧を取得します。

**パラメータ**:
| 名前 | 型 | 説明 |
|------|------|------|
| `path` | `string` | ディレクトリパス |

**戻り値**: `Vec<FileEntry>`

```typescript
interface FileEntry {
  name: string;
  path: string;
  isDirectory: boolean;
  size?: number;
}
```

---

### `select_mock_server_directory`

ディレクトリ選択ダイアログを表示します。

**パラメータ**: なし

**戻り値**: `Option<String>`

---

## イベント

Tauri イベントシステムで配信されるイベント。

### `tool-log`

ツール実行時のログ出力。

```typescript
interface LogEvent {
  type: "stdout" | "stderr";
  content: string;
  timestamp: string;
}
```

### `tool-status`

ツールのステータス変更。

```typescript
interface StatusEvent {
  status: "running" | "completed" | "failed";
  exitCode?: number;
}
```

### `level-up`

レベルアップイベント。

```typescript
interface LevelUpEvent {
  previousLevel: number;
  newLevel: number;
  newXp: number;
}
```

### `xp-gained`

XP 獲得イベント。

```typescript
interface XpGainedEvent {
  amount: number;
  source: string;
  totalXp: number;
}
```
