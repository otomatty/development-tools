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

