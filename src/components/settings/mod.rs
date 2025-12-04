pub mod account_settings;
pub mod app_info;
pub mod appearance_settings;
pub mod data_management;
pub mod notification_settings;
pub mod settings_reset;
pub mod sync_settings;

pub use account_settings::AccountSettings;
pub use app_info::AppInfoSection;
pub use appearance_settings::AppearanceSettings;
pub use data_management::DataManagement;
pub use notification_settings::NotificationSettings;
// SettingsPage is exported through pages/settings_page.rs
pub use settings_reset::SettingsResetSection;
pub use sync_settings::SyncSettings;
