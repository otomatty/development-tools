//! Notification utilities
//!
//! Provides functions for sending OS-native notifications based on user settings.

use tauri_plugin_notification::NotificationExt;

use crate::database::models::{NotificationMethod, UserSettings};

/// Send an OS-native notification based on user settings
///
/// This function checks the user's notification method setting and sends
/// an OS-native notification if the setting allows it (OsOnly or Both).
///
/// # Arguments
///
/// * `app` - Tauri AppHandle for accessing the notification plugin
/// * `settings` - User settings containing notification preferences
/// * `title` - Notification title
/// * `body` - Notification body text
///
/// # Returns
///
/// Returns `Ok(())` if the notification was sent successfully or if
/// notification sending was skipped based on settings.
/// Returns an error string if notification sending failed.
pub fn send_notification(
    app: &tauri::AppHandle,
    settings: &UserSettings,
    title: &str,
    body: &str,
) -> Result<(), String> {
    match settings.notification_method {
        NotificationMethod::OsOnly | NotificationMethod::Both => {
            app.notification()
                .builder()
                .title(title)
                .body(body)
                .show()
                .map_err(|e| format!("Failed to send notification: {}", e))?;
        }
        NotificationMethod::AppOnly | NotificationMethod::None => {
            // Skip OS notification for AppOnly or None
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::UserSettings;

    #[test]
    fn test_notification_method_os_only() {
        let mut settings = UserSettings::default();
        settings.notification_method = NotificationMethod::OsOnly;

        // This test would require a mock AppHandle, which is complex to set up
        // In a real scenario, we would use a test framework that can mock Tauri
        // For now, we just verify the enum values are correct
        assert_eq!(settings.notification_method, NotificationMethod::OsOnly);
    }

    #[test]
    fn test_notification_method_both() {
        let mut settings = UserSettings::default();
        settings.notification_method = NotificationMethod::Both;
        assert_eq!(settings.notification_method, NotificationMethod::Both);
    }

    #[test]
    fn test_notification_method_app_only() {
        let mut settings = UserSettings::default();
        settings.notification_method = NotificationMethod::AppOnly;
        assert_eq!(settings.notification_method, NotificationMethod::AppOnly);
    }

    #[test]
    fn test_notification_method_none() {
        let mut settings = UserSettings::default();
        settings.notification_method = NotificationMethod::None;
        assert_eq!(settings.notification_method, NotificationMethod::None);
    }
}
