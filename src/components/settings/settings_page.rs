//! Settings page component
//!
//! Main settings page with accordion-style sections.

use leptos::prelude::*;
use std::collections::HashSet;

use crate::components::icons::Icon;
use crate::components::settings::{AccountSettings, AppearanceSettings, AppInfoSection, DataManagement, NotificationSettings, SettingsResetSection, SyncSettings};
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

/// Accordion section component with icon
#[component]
fn AccordionSection(
    title: String,
    icon: &'static str,
    is_expanded: Memo<bool>,
    toggle: impl Fn() + 'static + Clone + Send + Sync,
    children: Children,
    #[prop(optional)] max_height: Option<&'static str>,
) -> impl IntoView
{
    let max_height = max_height.unwrap_or("500px");
    let section_id = format!("accordion-section-{}", title.replace(" ", "-").to_lowercase());
    let content_id = format!("{}-content", section_id);
    
    let toggle_click = toggle.clone();
    let toggle_key = toggle.clone();
    
    view! {
        <div class="bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 shadow-lg overflow-hidden transition-all duration-300 hover:border-gm-accent-cyan/40 hover:shadow-gm-accent-cyan/10">
            <button
                class="w-full px-6 py-4 flex items-center justify-between text-left hover:bg-gm-accent-cyan/10 transition-all duration-200 group focus:outline-none focus:ring-2 focus:ring-inset focus:ring-gm-accent-cyan"
                type="button"
                on:click=move |_| toggle_click()
                on:keydown=move |ev| {
                    if ev.key() == "Enter" || ev.key() == " " {
                        ev.prevent_default();
                        toggle_key();
                    }
                }
                aria-expanded=move || is_expanded.get()
                aria-controls=content_id.clone()
                id=section_id.clone()
            >
                <div class="flex items-center gap-3">
                    <span class="text-gm-accent-cyan group-hover:scale-110 transition-transform duration-200">
                        <Icon name=icon class="w-5 h-5".to_string() />
                    </span>
                    <span class="text-lg font-gaming font-bold text-white group-hover:text-gm-accent-cyan transition-colors duration-200">
                        {title}
                    </span>
                </div>
                <span 
                    class="text-gm-accent-cyan transition-transform duration-300 ease-in-out"
                    style:transform=move || if is_expanded.get() { "rotate(180deg)" } else { "rotate(0deg)" }
                    aria-hidden="true"
                >
                    <Icon name="chevron-down" class="w-5 h-5".to_string() />
                </span>
            </button>
            <div 
                id=content_id
                role="region"
                aria-labelledby=section_id
                class="overflow-hidden transition-all duration-300 ease-in-out"
                style:max-height=move || if is_expanded.get() { max_height } else { "0px" }
                style:opacity=move || if is_expanded.get() { "1" } else { "0" }
            >
                <div class="px-6 pb-6 pt-2">
                    {children()}
                </div>
            </div>
        </div>
    }
}

/// Settings page component
#[component]
pub fn SettingsPage(
    auth_state: ReadSignal<AuthState>,
    set_auth_state: WriteSignal<AuthState>,
    set_current_page: WriteSignal<AppPage>,
) -> impl IntoView
{
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
    let account_expanded = Memo::new(move |_| expanded_sections.get().contains(&SettingsSection::Account));
    let notification_expanded = Memo::new(move |_| expanded_sections.get().contains(&SettingsSection::Notification));
    let sync_expanded = Memo::new(move |_| expanded_sections.get().contains(&SettingsSection::Sync));
    let appearance_expanded = Memo::new(move |_| expanded_sections.get().contains(&SettingsSection::Appearance));
    let data_management_expanded = Memo::new(move |_| expanded_sections.get().contains(&SettingsSection::DataManagement));
    let app_info_expanded = Memo::new(move |_| expanded_sections.get().contains(&SettingsSection::AppInfo));

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
                    is_expanded=account_expanded
                    toggle=move || toggle_section(SettingsSection::Account)
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
                    is_expanded=notification_expanded
                    toggle=move || toggle_section(SettingsSection::Notification)
                    max_height="1000px"
                >
                    <NotificationSettings />
                </AccordionSection>

                // Sync Settings Section
                <AccordionSection
                    title="同期設定".to_string()
                    icon="refresh-cw"
                    is_expanded=sync_expanded
                    toggle=move || toggle_section(SettingsSection::Sync)
                    max_height="1000px"
                >
                    <SyncSettings />
                </AccordionSection>

                // Appearance Settings Section
                <AccordionSection
                    title="外観設定".to_string()
                    icon="palette"
                    is_expanded=appearance_expanded
                    toggle=move || toggle_section(SettingsSection::Appearance)
                    max_height="500px"
                >
                    <AppearanceSettings />
                </AccordionSection>

                // Data Management Section
                <AccordionSection
                    title="データ管理".to_string()
                    icon="database"
                    is_expanded=data_management_expanded
                    toggle=move || toggle_section(SettingsSection::DataManagement)
                    max_height="1200px"
                >
                    <DataManagement />
                </AccordionSection>

                // App Info Section
                <AccordionSection
                    title="アプリ情報".to_string()
                    icon="info"
                    is_expanded=app_info_expanded
                    toggle=move || toggle_section(SettingsSection::AppInfo)
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
