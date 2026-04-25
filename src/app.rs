use futures::join;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::pages::{
    HomePage, ProjectDashboardPage, ProjectsPage, SettingsPage, XpHistoryPage,
};
use crate::components::ui::feedback::OfflineBanner;
use crate::components::Sidebar;
use crate::contexts::{AnimationContext, NetworkStatusProvider};
use crate::tauri_api;
use crate::types::{AppPage, AuthState};

#[component]
pub fn App() -> impl IntoView {
    // ページ状態
    let (current_page, set_current_page) = signal(AppPage::Home);

    // 認証状態（SettingsPageで使用）
    let (auth_state, set_auth_state) = signal(AuthState::default());

    // アニメーション状態（グローバル）
    let animation_context = AnimationContext::new(true);
    provide_context(animation_context);

    // 認証状態とアニメーション設定を並列で初期化
    {
        let animation_ctx = animation_context;
        spawn_local(async move {
            // 認証状態と設定を並列で取得
            let (auth_result, settings_result) =
                join!(tauri_api::get_auth_state(), tauri_api::get_settings());

            // 認証状態を処理
            match auth_result {
                Ok(state) => {
                    set_auth_state.set(state);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get auth state: {}", e).into());
                }
            }

            // 設定を処理してアニメーション状態を更新
            match settings_result {
                Ok(settings) => {
                    animation_ctx.set_enabled.set(settings.animations_enabled);
                }
                Err(e) => {
                    // ログインしていない場合はエラーが出るが、デフォルト値を使用
                    web_sys::console::log_1(
                        &format!("Settings not loaded (may not be logged in): {}", e).into(),
                    );
                }
            }
        });
    }

    view! {
        <NetworkStatusProvider>
            <div class=move || {
                let base = "flex h-screen bg-dt-bg";
                if animation_context.enabled.get() {
                    base.to_string()
                } else {
                    format!("{} no-animation", base)
                }
            }>
                // サイドバー
                <Sidebar
                    current_page=current_page
                    set_current_page=set_current_page
                />

                // メインコンテンツ
                <main class="flex-1 flex flex-col overflow-hidden">
                    // オフラインバナー（オフライン時のみ表示）
                    <OfflineBanner />

                    // ページに応じてコンテンツを表示
                    {move || match current_page.get() {
                        AppPage::Home => view! {
                            <HomePage set_current_page=set_current_page />
                        }.into_any(),

                        AppPage::Projects => view! {
                            <ProjectsPage set_current_page=set_current_page />
                        }.into_any(),

                        AppPage::ProjectDetail(project_id) => view! {
                            <ProjectDashboardPage project_id=project_id set_current_page=set_current_page />
                        }.into_any(),

                        AppPage::Settings => view! {
                            <SettingsPage
                                auth_state=auth_state
                                set_auth_state=set_auth_state
                                set_current_page=set_current_page
                            />
                        }.into_any(),

                        AppPage::XpHistory => view! {
                            <XpHistoryPage set_current_page=set_current_page />
                        }.into_any(),
                    }}
                </main>
            </div>
        </NetworkStatusProvider>
    }
}
