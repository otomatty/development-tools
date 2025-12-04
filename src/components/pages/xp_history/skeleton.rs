//! XP History Skeleton Component
//!
//! Loading skeleton for XP history page.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/xp_history/mod.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;

/// Loading skeleton for XP history items
#[component]
pub fn XpHistorySkeleton() -> impl IntoView {
    view! {
        <div class="space-y-3">
            {(0..5).map(|_| view! {
                <div class="flex items-center gap-4 p-4 bg-gm-bg-card/50 rounded-xl border border-slate-700/30 animate-pulse">
                    <div class="w-12 h-12 bg-slate-700/50 rounded-xl"></div>
                    <div class="flex-1 space-y-2">
                        <div class="h-4 bg-slate-700/50 rounded w-24"></div>
                        <div class="h-3 bg-slate-700/50 rounded w-48"></div>
                    </div>
                    <div class="h-6 bg-slate-700/50 rounded w-16"></div>
                </div>
            }).collect_view()}
        </div>
    }
}

/// Empty state component
#[component]
pub fn EmptyState() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center py-16 text-center">
            <div class="w-20 h-20 mb-6 flex items-center justify-center bg-slate-800/50 rounded-full text-4xl">
                "📜"
            </div>
            <h3 class="text-xl font-gaming font-bold text-dt-text mb-2">
                "まだ履歴がありません"
            </h3>
            <p class="text-dt-text-sub max-w-md">
                "GitHubで活動すると、ここにXP獲得履歴が表示されます。"
                <br />
                "コミット、PR作成、レビューなどでXPを獲得しましょう！"
            </p>
        </div>
    }
}
