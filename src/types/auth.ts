// Authentication-related types

/// 認証状態
export interface AuthState {
  isLoggedIn: boolean;
  user: UserInfo | null;
}

/// ユーザー情報
export interface UserInfo {
  id: number;
  githubId: number;
  username: string;
  avatarUrl: string | null;
  createdAt: string | null;
}

/// Device Flow開始時のレスポンス
export interface DeviceCodeResponse {
  deviceCode: string;
  userCode: string;
  verificationUri: string;
  expiresIn: number;
  interval: number;
}

/// Device Flowトークンポーリングのステータス
export type DeviceTokenStatus =
  | { status: 'pending' }
  | { status: 'success'; authState: AuthState }
  | { status: 'error'; message: string };

/// 認証切れイベントのペイロード
///
/// バックエンドが GitHub から 401 を受け取った際、または起動時のトークン検証で
/// 失効が確認された際に `auth-expired` イベントとして emit される。
/// 詳細は `src-tauri/src/auth/session.rs` を参照。
export interface AuthExpiredEvent {
  /// マシン可読な理由コード（"github_unauthorized", "startup_validation_failed" など）
  reason: string;
  /// UI に表示できる日本語メッセージ
  message: string;
}

/// GitHubユーザー
export interface GitHubUser {
  id: number;
  login: string;
  avatarUrl: string;
  name: string | null;
  bio: string | null;
  publicRepos: number;
  followers: number;
  following: number;
  createdAt: string;
}

