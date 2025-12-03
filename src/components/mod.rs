pub mod animated_emoji;
pub mod animation_context;
pub mod home;
pub mod icons;
pub mod issues;
pub mod log_viewer;
pub mod mock_server;
pub mod network_status;
pub mod result_view;
pub mod settings;
pub mod sidebar;
pub mod tool_detail;
pub mod ui;

// Legacy modules - kept for backward compatibility
// TODO: [DEBT] Remove these re-exports after migrating all usages to ui/ path
pub mod confirm_dialog;
pub mod dropdown_menu;
// TODO: [DEBT] skeleton module was removed, but declaration remained. Need to check if skeleton component is needed.
// pub mod skeleton;

pub use animated_emoji::{
    AnimatedEmoji, AnimatedEmojiWithIntensity, AnimationIntensity, EmojiType,
};
pub use animation_context::{
    use_animation_context, use_animation_context_or_default, AnimationContext,
};
pub use confirm_dialog::ConfirmDialog;
pub use dropdown_menu::{DropdownMenu, DropdownMenuDivider, DropdownMenuItem};
pub use home::{HomePage, XpHistoryPage};
pub use issues::{ProjectDashboard, ProjectsPage};
pub use log_viewer::LogViewer;
pub use mock_server::MockServerPage;
pub use network_status::{
    try_use_network_status, use_is_online, use_network_status, NetworkStatusProvider, OfflineBanner,
};
pub use result_view::ResultView;
pub use sidebar::Sidebar;
pub use tool_detail::ToolDetail;

// UI components (new paths)
pub use ui::{Card, CardVariant, Modal};
// Form components (new paths) - re-exported for backward compatibility
pub use ui::form::{LabeledToggle, OptionForm, ToggleSwitch, ToggleSwitchSize};
// Feedback components (new paths) - re-exported for backward compatibility
pub use ui::feedback::{InlineToast, SaveStatusIndicator, Toast, ToastType};
