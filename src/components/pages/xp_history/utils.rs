//! XP History Utility Functions
//!
//! Helper functions for XP history display formatting and styling.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/xp_history/mod.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

/// Get icon for action type
pub fn get_action_icon(action_type: &str) -> &'static str {
    match action_type {
        "commit" => "📝",
        "pull_request" => "🔀",
        "pull_request_merged" => "✅",
        "review" => "👀",
        "issue" => "📋",
        "issue_closed" => "✔️",
        "streak_bonus" => "🔥",
        "star" => "⭐",
        _ => "💫",
    }
}

/// Get display name for action type
pub fn get_action_display_name(action_type: &str) -> &'static str {
    match action_type {
        "commit" => "コミット",
        "pull_request" => "PR作成",
        "pull_request_merged" => "PRマージ",
        "review" => "レビュー",
        "issue" => "Issue作成",
        "issue_closed" => "Issueクローズ",
        "streak_bonus" => "ストリークボーナス",
        "star" => "スター獲得",
        _ => "その他",
    }
}

/// Get color class for action type
pub fn get_action_color_class(action_type: &str) -> &'static str {
    match action_type {
        "commit" => "text-blue-400",
        "pull_request" => "text-purple-400",
        "pull_request_merged" => "text-green-400",
        "review" => "text-yellow-400",
        "issue" => "text-orange-400",
        "issue_closed" => "text-emerald-400",
        "streak_bonus" => "text-red-400",
        "star" => "text-amber-400",
        _ => "text-gray-400",
    }
}

/// Format relative time from ISO8601 string
pub fn format_relative_time(created_at: &str) -> String {
    let now = js_sys::Date::new_0();
    let today = js_sys::Date::new_0();
    today.set_hours(0);
    today.set_minutes(0);
    today.set_seconds(0);
    today.set_milliseconds(0);

    let created_date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(created_at));
    let created_time = created_date.get_time();

    if created_time.is_nan() {
        return "不明".to_string();
    }

    let diff_ms = now.get_time() - created_time;
    let diff_days = (diff_ms / (1000.0 * 60.0 * 60.0 * 24.0)).floor() as i32;

    if diff_days == 0 {
        "今日".to_string()
    } else if diff_days == 1 {
        "昨日".to_string()
    } else if diff_days < 7 {
        format!("{}日前", diff_days)
    } else if diff_days < 30 {
        format!("{}週間前", diff_days / 7)
    } else if diff_days < 365 {
        format!("{}ヶ月前", diff_days / 30)
    } else {
        format!("{}年前", diff_days / 365)
    }
}

/// Format absolute time from ISO8601 string
pub fn format_absolute_time(created_at: &str) -> String {
    let created_date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(created_at));
    let created_time = created_date.get_time();

    if created_time.is_nan() {
        return "不明".to_string();
    }

    let year = created_date.get_full_year() as i32;
    let month = created_date.get_month() as i32 + 1;
    let day = created_date.get_date() as i32;
    let hours = created_date.get_hours() as i32;
    let minutes = created_date.get_minutes() as i32;

    format!(
        "{}/{:02}/{:02} {:02}:{:02}",
        year, month, day, hours, minutes
    )
}
