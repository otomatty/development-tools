// Network status types

/// 現在時刻をISO 8601形式で取得
export function getCurrentTimestamp(): string {
  return new Date().toISOString();
}

/// ネットワーク接続状態
export interface NetworkState {
  /// オンラインかどうか
  isOnline: boolean;
  /// 最終確認時刻 (ISO 8601形式)
  lastCheckedAt: string | null;
  /// 最後にオンラインになった時刻 (ISO 8601形式)
  lastOnlineAt: string | null;
}

/// 新しいネットワーク状態を作成
export function newNetworkState(isOnline: boolean): NetworkState {
  const now = getCurrentTimestamp();
  return {
    isOnline,
    lastCheckedAt: now,
    lastOnlineAt: isOnline ? now : null,
  };
}

/// オンライン状態に更新
export function setOnline(state: NetworkState): NetworkState {
  const now = getCurrentTimestamp();
  return {
    ...state,
    isOnline: true,
    lastCheckedAt: now,
    lastOnlineAt: now,
  };
}

/// オフライン状態に更新
export function setOffline(state: NetworkState): NetworkState {
  const now = getCurrentTimestamp();
  return {
    ...state,
    isOnline: false,
    lastCheckedAt: now,
    // lastOnlineAt は保持（最後にオンラインだった時刻を記録）
  };
}

/// デフォルトのネットワーク状態（オンラインと仮定）
export function defaultNetworkState(): NetworkState {
  return {
    isOnline: true,
    lastCheckedAt: null,
    lastOnlineAt: null,
  };
}

