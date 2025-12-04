//! Settings page component
//!
//! Main settings page with accordion-style sections.
//!
//! DEPENDENCY MAP:
//!
//! Parents:
//!   └─ src/components/pages/mod.rs
//! Dependencies:
//!   ├─ src/components/ui/accordion.rs
//!   ├─ src/components/icons.rs
//!   └─ src/components/settings/*.rs

use leptos::prelude::*;
use std::collections::HashSet;

use crate::components::icons::Icon;
use crate::components::settings::{
    AccountSettings, AppInfoSection, AppearanceSettings, DataManagement, NotificationSettings,
    SettingsResetSection, SyncSettings,
};
use crate::components::ui::AccordionSection;
use crate::types::{AppPage, AuthState};

/// Settings section enum
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum SettingsSection {
    Account,
    Notification,
    Sync,
    Appearance,
    DataManagement,
    AppInfo,
}

/// Settings page component
#[component]
pub fn SettingsPage(
    auth_state: ReadSignal<AuthState>,
    set_auth_state: WriteSignal<AuthState>,
    set_current_page: WriteSignal<AppPage>,
) -> impl IntoView {
    let (expanded_sections, set_expanded_sections) = signal({
        let mut set = HashSet::new();
        set.insert(SettingsSection::Account);
        set
    });

    let toggle_section = move |section: SettingsSection| {
        set_expanded_sections.update(|sections| {
            if sections.contains(&section) {
                sections.remove(&section);
            } else {
                sections.insert(section);
            }
        });
    };

    // Create signals for each section's expanded state
    let account_expanded =
        Signal::derive(move || expanded_sections.get().contains(&SettingsSection::Account));
    let notification_expanded = Signal::derive(move || {
        expanded_sections
            .get()
            .contains(&SettingsSection::Notification)
    });
    let sync_expanded =
        Signal::derive(move || expanded_sections.get().contains(&SettingsSection::Sync));
    let appearance_expanded = Signal::derive(move || {
        expanded_sections
            .get()
            .contains(&SettingsSection::Appearance)
    });
    let data_management_expanded = Signal::derive(move || {
        expanded_sections
            .get()
            .contains(&SettingsSection::DataManagement)
    });
    let app_info_expanded =
        Signal::derive(move || expanded_sections.get().contains(&SettingsSection::AppInfo));

    view! {
        <div class="flex-1 overflow-y-auto p-6">
            <h1 class="text-3xl font-gaming font-bold text-white mb-6 flex items-center gap-3">
                <Icon name="settings" class="w-8 h-8 text-gm-accent-cyan".to_string() />
                "SETTINGS"
            </h1>

            <div class="space-y-4">
                // Account Settings Section
                <AccordionSection
                    title="アカウント設定".to_string()
                    icon="user"
                    expanded=account_expanded
                    on_toggle=move || toggle_section(SettingsSection::Account)
                    max_height="1000px"
                >
                    <AccountSettings
                        auth_state=auth_state
                        set_auth_state=set_auth_state
                        set_current_page=set_current_page
                    />
                </AccordionSection>

                // Notification Settings Section
                <AccordionSection
                    title="通知設定".to_string()
                    icon="bell"
                    expanded=notification_expanded
                    on_toggle=move || toggle_section(SettingsSection::Notification)
                    max_height="1000px"
                >
                    <NotificationSettings />
                </AccordionSection>

                // Sync Settings Section
                <AccordionSection
                    title="同期設定".to_string()
                    icon="refresh-cw"
                    expanded=sync_expanded
                    on_toggle=move || toggle_section(SettingsSection::Sync)
                    max_height="1000px"
                >
                    <SyncSettings />
                </AccordionSection>

                // Appearance Settings Section
                <AccordionSection
                    title="外観設定".to_string()
                    icon="palette"
                    expanded=appearance_expanded
                    on_toggle=move || toggle_section(SettingsSection::Appearance)
                    max_height="500px"
                >
                    <AppearanceSettings />
                </AccordionSection>

                // Data Management Section
                <AccordionSection
                    title="データ管理".to_string()
                    icon="database"
                    expanded=data_management_expanded
                    on_toggle=move || toggle_section(SettingsSection::DataManagement)
                    max_height="1200px"
                >
                    <DataManagement />
                </AccordionSection>

                // App Info Section
                <AccordionSection
                    title="アプリ情報".to_string()
                    icon="info"
                    expanded=app_info_expanded
                    on_toggle=move || toggle_section(SettingsSection::AppInfo)
                    max_height="600px"
                >
                    <AppInfoSection />
                </AccordionSection>

                // Settings Reset Section (not in accordion)
                <div class="mt-6">
                    <SettingsResetSection />
                </div>
            </div>
        </div>
    }
}
