//! Settings Page Utility Functions
//!
//! Helper functions for the settings page.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/settings/mod.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

/// Settings section identifiers
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsSection {
    Account,
    Notification,
    Sync,
    Appearance,
    DataManagement,
    AppInfo,
}

/// Get section icon name
pub fn get_section_icon(section: SettingsSection) -> &'static str {
    match section {
        SettingsSection::Account => "user",
        SettingsSection::Notification => "bell",
        SettingsSection::Sync => "refresh-cw",
        SettingsSection::Appearance => "palette",
        SettingsSection::DataManagement => "database",
        SettingsSection::AppInfo => "info",
    }
}

/// Get section title
pub fn get_section_title(section: SettingsSection) -> &'static str {
    match section {
        SettingsSection::Account => "アカウント設定",
        SettingsSection::Notification => "通知設定",
        SettingsSection::Sync => "同期設定",
        SettingsSection::Appearance => "外観設定",
        SettingsSection::DataManagement => "データ管理",
        SettingsSection::AppInfo => "アプリ情報",
    }
}

/// Get section max height
pub fn get_section_max_height(section: SettingsSection) -> &'static str {
    match section {
        SettingsSection::Account => "1000px",
        SettingsSection::Notification => "1000px",
        SettingsSection::Sync => "1000px",
        SettingsSection::Appearance => "500px",
        SettingsSection::DataManagement => "1200px",
        SettingsSection::AppInfo => "600px",
    }
}
