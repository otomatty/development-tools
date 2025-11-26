//! Login card component
//!
//! Displays when user is not logged in.
//! Supports GitHub Device Flow authentication.

use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

/// Login card state
#[derive(Clone, PartialEq)]
pub enum LoginState {
    /// Initial state - show login button
    Initial,
    /// Starting device flow
    Starting,
    /// Waiting for user to enter code
    WaitingForCode {
        user_code: String,
        verification_uri: String,
        expires_in: i64,
    },
    /// Polling for token
    Polling,
    /// Error occurred
    Error(String),
}

impl Default for LoginState {
    fn default() -> Self {
        Self::Initial
    }
}

/// Login card for unauthenticated users with Device Flow support
#[component]
pub fn LoginCard(
    login_state: ReadSignal<LoginState>,
    on_login: Callback<ev::MouseEvent>,
    on_cancel: Callback<ev::MouseEvent>,
    on_open_url: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-center min-h-[60vh]">
            <div class="text-center p-12 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 shadow-neon-cyan max-w-md w-full">
                // Logo/Icon
                <div class="mb-8">
                    <div class="w-24 h-24 mx-auto bg-gradient-to-br from-gm-accent-cyan to-gm-accent-purple rounded-2xl flex items-center justify-center shadow-neon-cyan">
                        <span class="text-5xl">"🎮"</span>
                    </div>
                </div>

                // Dynamic content based on login state
                {move || {
                    let state = login_state.get();
                    match state {
                        LoginState::Initial => view! {
                            <InitialView on_login=on_login />
                        }.into_any(),
                        LoginState::Starting => view! {
                            <StartingView />
                        }.into_any(),
                        LoginState::WaitingForCode { user_code, verification_uri, expires_in: _ } => view! {
                            <WaitingForCodeView
                                user_code=user_code.clone()
                                verification_uri=verification_uri.clone()
                                on_cancel=on_cancel
                                on_open_url=on_open_url
                            />
                        }.into_any(),
                        LoginState::Polling => view! {
                            <PollingView on_cancel=on_cancel />
                        }.into_any(),
                        LoginState::Error(message) => view! {
                            <ErrorView message=message.clone() on_retry=on_login />
                        }.into_any(),
                    }
                }}
            </div>
        </div>
    }
}

/// Initial view with login button
#[component]
fn InitialView(on_login: Callback<ev::MouseEvent>) -> impl IntoView {
    view! {
        <>
            // Title
            <h2 class="text-3xl font-gaming font-bold text-white mb-4">
                "Level Up Your Dev Game"
            </h2>

            // Description
            <p class="text-dt-text-sub mb-8 font-gaming-body text-lg">
                "Track your GitHub activity, earn XP, unlock badges, and become a legendary developer."
            </p>

            // Features list
            <div class="text-left mb-8 space-y-3">
                <div class="flex items-center gap-3 text-dt-text-sub">
                    <span class="text-gm-success">"✓"</span>
                    <span>"Track commits, PRs, and reviews"</span>
                </div>
                <div class="flex items-center gap-3 text-dt-text-sub">
                    <span class="text-gm-success">"✓"</span>
                    <span>"Earn XP and level up"</span>
                </div>
                <div class="flex items-center gap-3 text-dt-text-sub">
                    <span class="text-gm-success">"✓"</span>
                    <span>"Unlock achievement badges"</span>
                </div>
                <div class="flex items-center gap-3 text-dt-text-sub">
                    <span class="text-gm-success">"✓"</span>
                    <span>"Maintain your commit streak"</span>
                </div>
            </div>

            // Login button
            <button
                class="w-full py-4 px-6 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white font-gaming font-bold text-lg rounded-xl hover:opacity-90 transition-all duration-200 shadow-neon-cyan flex items-center justify-center gap-3"
                on:click=move |e| on_login.run(e)
            >
                <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                </svg>
                "Connect with GitHub"
            </button>

            // Note
            <p class="mt-6 text-xs text-dt-text-sub">
                "We only request read access to your public activity."
            </p>
        </>
    }
}

/// Loading view while starting device flow
#[component]
fn StartingView() -> impl IntoView {
    view! {
        <>
            <h2 class="text-2xl font-gaming font-bold text-white mb-4">
                "Starting Authentication..."
            </h2>
            <div class="flex justify-center mb-8">
                <div class="animate-spin rounded-full h-12 w-12 border-4 border-gm-accent-cyan border-t-transparent"></div>
            </div>
            <p class="text-dt-text-sub">
                "Please wait while we set up the connection."
            </p>
        </>
    }
}

/// View showing the device code for user to enter
#[component]
fn WaitingForCodeView(
    user_code: String,
    verification_uri: String,
    on_cancel: Callback<ev::MouseEvent>,
    on_open_url: Callback<String>,
) -> impl IntoView {
    let verification_uri_clone = verification_uri.clone();
    let user_code_for_copy = user_code.clone();
    
    // State for copy feedback
    let (copied, set_copied) = signal(false);
    
    // Copy to clipboard function
    let copy_to_clipboard = move |_| {
        let code = user_code_for_copy.clone();
        let set_copied = set_copied.clone();
        
        spawn_local(async move {
            if let Some(window) = web_sys::window() {
                let clipboard = window.navigator().clipboard();
                let promise = clipboard.write_text(&code);
                if wasm_bindgen_futures::JsFuture::from(promise).await.is_ok() {
                    set_copied.set(true);
                    // Reset after 2 seconds
                    gloo_timers::callback::Timeout::new(2000, move || {
                        set_copied.set(false);
                    }).forget();
                }
            }
        });
    };
    
    view! {
        <>
            <h2 class="text-2xl font-gaming font-bold text-white mb-4">
                "Enter This Code on GitHub"
            </h2>

            // User code display with copy button
            <div class="bg-gm-bg-dark/50 rounded-xl p-6 mb-6 border border-gm-accent-purple/30">
                <p class="text-sm text-dt-text-sub mb-2">"Your code:"</p>
                <div class="flex items-center justify-center gap-3">
                    <div class="text-4xl font-mono font-bold text-gm-accent-cyan tracking-widest select-all">
                        {user_code}
                    </div>
                    <button
                        class="p-2 rounded-lg bg-gm-accent-cyan/20 hover:bg-gm-accent-cyan/30 transition-colors border border-gm-accent-cyan/30 group"
                        on:click=copy_to_clipboard
                        title="Copy to clipboard"
                    >
                        {move || if copied.get() {
                            view! {
                                // Checkmark icon when copied
                                <svg class="w-6 h-6 text-gm-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                                </svg>
                            }.into_any()
                        } else {
                            view! {
                                // Copy icon
                                <svg class="w-6 h-6 text-gm-accent-cyan group-hover:text-white transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
                                </svg>
                            }.into_any()
                        }}
                    </button>
                </div>
                // Copy feedback message
                <div class="h-6 mt-2">
                    {move || if copied.get() {
                        view! {
                            <p class="text-sm text-gm-success animate-fade-in">"✓ Copied to clipboard!"</p>
                        }.into_any()
                    } else {
                        view! { <p></p> }.into_any()
                    }}
                </div>
            </div>

            // Instructions
            <div class="text-left mb-6 space-y-4">
                <div class="flex items-start gap-3">
                    <span class="flex-shrink-0 w-6 h-6 bg-gm-accent-cyan/20 rounded-full flex items-center justify-center text-gm-accent-cyan text-sm font-bold">"1"</span>
                    <span class="text-dt-text-sub">"Click the button below to open GitHub"</span>
                </div>
                <div class="flex items-start gap-3">
                    <span class="flex-shrink-0 w-6 h-6 bg-gm-accent-cyan/20 rounded-full flex items-center justify-center text-gm-accent-cyan text-sm font-bold">"2"</span>
                    <span class="text-dt-text-sub">"Enter the code shown above"</span>
                </div>
                <div class="flex items-start gap-3">
                    <span class="flex-shrink-0 w-6 h-6 bg-gm-accent-cyan/20 rounded-full flex items-center justify-center text-gm-accent-cyan text-sm font-bold">"3"</span>
                    <span class="text-dt-text-sub">"Authorize the application"</span>
                </div>
            </div>

            // Open GitHub button
            <button
                class="w-full py-4 px-6 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white font-gaming font-bold text-lg rounded-xl hover:opacity-90 transition-all duration-200 shadow-neon-cyan flex items-center justify-center gap-3 mb-4"
                on:click=move |_| on_open_url.run(verification_uri_clone.clone())
            >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/>
                </svg>
                "Open GitHub"
            </button>

            // Cancel button
            <button
                class="w-full py-3 px-6 text-dt-text-sub hover:text-white transition-colors"
                on:click=move |e| on_cancel.run(e)
            >
                "Cancel"
            </button>

            // URL hint
            <p class="mt-4 text-xs text-dt-text-sub">
                "Or visit: "
                <span class="text-gm-accent-cyan">{verification_uri}</span>
            </p>
        </>
    }
}

/// View showing polling status
#[component]
fn PollingView(on_cancel: Callback<ev::MouseEvent>) -> impl IntoView {
    view! {
        <>
            <h2 class="text-2xl font-gaming font-bold text-white mb-4">
                "Waiting for Authorization..."
            </h2>

            <div class="flex justify-center mb-6">
                <div class="animate-spin rounded-full h-12 w-12 border-4 border-gm-accent-purple border-t-transparent"></div>
            </div>

            <p class="text-dt-text-sub mb-6">
                "Complete the authorization in your browser. This page will update automatically."
            </p>

            // Cancel button
            <button
                class="py-3 px-6 text-dt-text-sub hover:text-white transition-colors"
                on:click=move |e| on_cancel.run(e)
            >
                "Cancel"
            </button>
        </>
    }
}

/// Error view with retry option
#[component]
fn ErrorView(message: String, on_retry: Callback<ev::MouseEvent>) -> impl IntoView {
    view! {
        <>
            <div class="text-gm-error text-5xl mb-4">"⚠️"</div>
            
            <h2 class="text-2xl font-gaming font-bold text-white mb-4">
                "Authentication Failed"
            </h2>

            <p class="text-dt-text-sub mb-6">
                {message}
            </p>

            // Retry button
            <button
                class="w-full py-4 px-6 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white font-gaming font-bold text-lg rounded-xl hover:opacity-90 transition-all duration-200 shadow-neon-cyan"
                on:click=move |e| on_retry.run(e)
            >
                "Try Again"
            </button>
        </>
    }
}
