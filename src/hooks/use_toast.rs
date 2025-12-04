//! Toast hooks
//!
//! トースト表示の管理用カスタムフック。
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   - src/hooks/mod.rs
//!   - src/components/ (various components)
//! Dependencies:
//!   - leptos

use leptos::prelude::*;

/// トーストメッセージの種類
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastType {
    /// 成功メッセージ
    #[default]
    Success,
    /// エラーメッセージ
    Error,
    /// 警告メッセージ
    Warning,
    /// 情報メッセージ
    Info,
}

/// トーストメッセージ
#[derive(Clone, Debug)]
pub struct ToastMessage {
    /// メッセージ内容
    pub message: String,
    /// トーストの種類
    pub toast_type: ToastType,
    /// 表示時間（ミリ秒）
    pub duration_ms: u32,
}

impl ToastMessage {
    /// 新しいトーストメッセージを作成
    pub fn new(message: impl Into<String>, toast_type: ToastType) -> Self {
        Self {
            message: message.into(),
            toast_type,
            duration_ms: 3000, // デフォルト3秒
        }
    }

    /// 成功メッセージを作成
    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Success)
    }

    /// エラーメッセージを作成
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Error)
    }

    /// 警告メッセージを作成
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Warning)
    }

    /// 情報メッセージを作成
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Info)
    }

    /// 表示時間を設定
    pub fn with_duration(mut self, duration_ms: u32) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

/// トースト管理のためのフック戻り値
#[derive(Clone, Copy)]
pub struct UseToast {
    /// 現在のトーストメッセージ
    pub current: ReadSignal<Option<ToastMessage>>,
    /// トーストが表示中かどうか
    pub is_visible: Signal<bool>,
    /// トーストを表示
    show: WriteSignal<Option<ToastMessage>>,
}

impl UseToast {
    /// 新しいトーストを表示
    pub fn show(&self, message: ToastMessage) {
        self.show.set(Some(message));
    }

    /// 成功トーストを表示
    pub fn success(&self, message: impl Into<String>) {
        self.show(ToastMessage::success(message));
    }

    /// エラートーストを表示
    pub fn error(&self, message: impl Into<String>) {
        self.show(ToastMessage::error(message));
    }

    /// 警告トーストを表示
    pub fn warning(&self, message: impl Into<String>) {
        self.show(ToastMessage::warning(message));
    }

    /// 情報トーストを表示
    pub fn info(&self, message: impl Into<String>) {
        self.show(ToastMessage::info(message));
    }

    /// トーストを非表示
    pub fn hide(&self) {
        self.show.set(None);
    }
}

/// トースト管理フック
///
/// トースト表示の状態管理を行うフック。
///
/// # Example
///
/// ```rust,ignore
/// let toast = use_toast();
///
/// // 成功メッセージを表示
/// toast.success("保存しました");
///
/// // エラーメッセージを表示
/// toast.error("エラーが発生しました");
///
/// // カスタムメッセージを表示
/// toast.show(ToastMessage::new("カスタムメッセージ", ToastType::Info).with_duration(5000));
/// ```
pub fn use_toast() -> UseToast {
    let (current, show) = signal(None::<ToastMessage>);

    let is_visible = Signal::derive(move || current.get().is_some());

    UseToast {
        current,
        is_visible,
        show,
    }
}

/// トーストコンテキスト（グローバルトースト用）
#[derive(Clone, Copy)]
pub struct ToastContext {
    /// 現在のトーストメッセージ
    pub current: ReadSignal<Option<ToastMessage>>,
    /// トーストを表示
    show: WriteSignal<Option<ToastMessage>>,
}

impl ToastContext {
    /// 新しいトーストコンテキストを作成
    pub fn new() -> Self {
        let (current, show) = signal(None::<ToastMessage>);
        Self { current, show }
    }

    /// コンテキストを提供
    pub fn provide(self) {
        provide_context(self);
    }

    /// 新しいトーストを表示
    pub fn show(&self, message: ToastMessage) {
        self.show.set(Some(message));
    }

    /// 成功トーストを表示
    pub fn success(&self, message: impl Into<String>) {
        self.show(ToastMessage::success(message));
    }

    /// エラートーストを表示
    pub fn error(&self, message: impl Into<String>) {
        self.show(ToastMessage::error(message));
    }

    /// トーストを非表示
    pub fn hide(&self) {
        self.show.set(None);
    }
}

impl Default for ToastContext {
    fn default() -> Self {
        Self::new()
    }
}

/// グローバルトーストコンテキストを取得
pub fn use_toast_context() -> Option<ToastContext> {
    use_context::<ToastContext>()
}

/// グローバルトーストコンテキストを取得（必須版）
pub fn use_toast_context_or_panic() -> ToastContext {
    use_context::<ToastContext>().expect("ToastContext must be provided")
}
