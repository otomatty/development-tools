//! Settings page component
//!
//! Main settings page with accordion-style sections.

use leptos::prelude::*;
use std::collections::HashSet;

use crate::components::settings::AccountSettings;
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

/// Accordion section component
#[component]
fn AccordionSection(
    title: String,
    is_expanded: Memo<bool>,
    toggle: impl Fn() + 'static + Clone + Send + Sync,
    children: Children,
    #[prop(optional)] max_height: Option<&'static str>,
) -> impl IntoView
{
    let max_height = max_height.unwrap_or("500px");
    
    view! {
        <div class="bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 shadow-lg overflow-hidden">
            <button
                class="w-full px-6 py-4 flex items-center justify-between text-left hover:bg-gm-accent-cyan/10 transition-colors"
                on:click=move |_| toggle()
            >
                <span class="text-lg font-gaming font-bold text-white">
                    {title}
                </span>
                <span 
                    class="text-gm-accent-cyan transition-transform duration-300 inline-block"
                    style:transform=move || if is_expanded.get() { "rotate(180deg)" } else { "rotate(0deg)" }
                >
                    "▼"
                </span>
            </button>
            <div 
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
            <h1 class="text-3xl font-gaming font-bold text-white mb-6">
                "SETTINGS"
            </h1>

            <div class="space-y-4">
                // Account Settings Section
                <AccordionSection
                    title="アカウント設定".to_string()
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

                // Notification Settings Section (placeholder)
                <AccordionSection
                    title="通知設定".to_string()
                    is_expanded=notification_expanded
                    toggle=move || toggle_section(SettingsSection::Notification)
                >
                    <div class="text-dt-text-sub">
                        "Coming soon..."
                    </div>
                </AccordionSection>

                // Sync Settings Section (placeholder)
                <AccordionSection
                    title="同期設定".to_string()
                    is_expanded=sync_expanded
                    toggle=move || toggle_section(SettingsSection::Sync)
                >
                    <div class="text-dt-text-sub">
                        "Coming soon..."
                    </div>
                </AccordionSection>

                // Appearance Settings Section (placeholder)
                <AccordionSection
                    title="外観設定".to_string()
                    is_expanded=appearance_expanded
                    toggle=move || toggle_section(SettingsSection::Appearance)
                >
                    <div class="text-dt-text-sub">
                        "Coming soon..."
                    </div>
                </AccordionSection>

                // Data Management Section (placeholder)
                <AccordionSection
                    title="データ管理".to_string()
                    is_expanded=data_management_expanded
                    toggle=move || toggle_section(SettingsSection::DataManagement)
                >
                    <div class="text-dt-text-sub">
                        "Coming soon..."
                    </div>
                </AccordionSection>

                // App Info Section (placeholder)
                <AccordionSection
                    title="アプリケーション設定".to_string()
                    is_expanded=app_info_expanded
                    toggle=move || toggle_section(SettingsSection::AppInfo)
                >
                    <div class="text-dt-text-sub">
                        "Coming soon..."
                    </div>
                </AccordionSection>
            </div>
        </div>
    }
}

