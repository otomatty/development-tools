pub mod account_settings;
pub mod app_info;
pub mod appearance_settings;
pub mod data_management;
pub mod notification_settings;
pub mod settings_page;
pub mod settings_reset;
pub mod sync_settings;

pub use account_settings::AccountSettings;
pub use app_info::AppInfoSection;
pub use appearance_settings::AppearanceSettings;
pub use data_management::DataManagement;
pub use notification_settings::NotificationSettings;
pub use settings_page::SettingsPage;
pub use settings_reset::SettingsResetSection;
pub use sync_settings::SyncSettings;

// Re-export from new locations for backward compatibility
// TODO: [DEBT] Update usages to import directly from ui/feedback and ui/form
pub use super::ui::feedback::{InlineToast, SaveStatusIndicator, Toast, ToastType};
pub use super::ui::form::{LabeledToggle, ToggleSwitch, ToggleSwitchSize};
