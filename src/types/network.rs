//! Network status types
//!
//! Types for tracking network connectivity state.

use serde::{Deserialize, Serialize};

/// 現在時刻をISO 8601形式で取得（WASM互換）
fn get_current_timestamp() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // 非WASM環境（テスト・LSP解析用）: std::timeから現在時刻を生成
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs();
        // 簡易的なISO 8601形式（UTC）
        let days_since_epoch = secs / 86400;
        let time_of_day = secs % 86400;
        let hours = time_of_day / 3600;
        let minutes = (time_of_day % 3600) / 60;
        let seconds = time_of_day % 60;
        // 1970-01-01からの日数を年月日に変換（簡易計算）
        let years = 1970 + (days_since_epoch / 365);
        format!("{:04}-01-01T{:02}:{:02}:{:02}Z", years, hours, minutes, seconds)
    }
}

/// ネットワーク接続状態
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkState {
    /// オンラインかどうか
    pub is_online: bool,
    /// 最終確認時刻 (ISO 8601形式)
    pub last_checked_at: Option<String>,
    /// 最後にオンラインになった時刻 (ISO 8601形式)
    pub last_online_at: Option<String>,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            // デフォルトはオンラインと仮定（楽観的フォールバック）
            is_online: true,
            last_checked_at: None,
            last_online_at: None,
        }
    }
}

impl NetworkState {
    /// 新しいネットワーク状態を作成
    pub fn new(is_online: bool) -> Self {
        let now = get_current_timestamp();
        Self {
            is_online,
            last_checked_at: Some(now.clone()),
            last_online_at: if is_online { Some(now) } else { None },
        }
    }

    /// オンライン状態に更新
    pub fn set_online(&mut self) {
        let now = get_current_timestamp();
        self.is_online = true;
        self.last_checked_at = Some(now.clone());
        self.last_online_at = Some(now);
    }

    /// オフライン状態に更新
    pub fn set_offline(&mut self) {
        let now = get_current_timestamp();
        self.is_online = false;
        self.last_checked_at = Some(now);
        // last_online_at は保持（最後にオンラインだった時刻を記録）
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-001: 初期状態の取得（オンライン）
    #[test]
    fn test_new_online_state() {
        let state = NetworkState::new(true);
        assert!(state.is_online);
        assert!(state.last_checked_at.is_some());
        assert!(state.last_online_at.is_some());
    }

    // TC-002: 初期状態の取得（オフライン）
    #[test]
    fn test_new_offline_state() {
        let state = NetworkState::new(false);
        assert!(!state.is_online);
        assert!(state.last_checked_at.is_some());
        assert!(state.last_online_at.is_none());
    }

    // TC-003: オフラインへの移行
    #[test]
    fn test_set_offline() {
        let mut state = NetworkState::new(true);
        let original_online_at = state.last_online_at.clone();
        
        state.set_offline();
        
        assert!(!state.is_online);
        assert!(state.last_checked_at.is_some());
        // last_online_at は保持される
        assert_eq!(state.last_online_at, original_online_at);
    }

    // TC-004: オンラインへの復帰
    #[test]
    fn test_set_online() {
        let mut state = NetworkState::new(false);
        assert!(state.last_online_at.is_none());
        
        state.set_online();
        
        assert!(state.is_online);
        assert!(state.last_checked_at.is_some());
        assert!(state.last_online_at.is_some());
    }

    // TC-007: フォールバック動作（デフォルト値）
    #[test]
    fn test_default_assumes_online() {
        let state = NetworkState::default();
        assert!(state.is_online);
    }
}
