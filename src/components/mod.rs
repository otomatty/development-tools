pub mod animated_emoji;
pub mod features;
pub mod home;
pub mod icons;
pub mod issues;
pub mod pages;
pub mod settings;
pub mod sidebar;
pub mod ui;

// Legacy modules - kept for backward compatibility
// TODO: [DEBT] Remove these re-exports after migrating all usages to ui/ path
pub mod confirm_dialog;
pub mod dropdown_menu;
// TODO: [DEBT] animation_context and network_status modules are deprecated.
// Use crate::contexts and crate::hooks instead.
// These are kept for backward compatibility during migration.
pub mod animation_context;
pub mod network_status;

pub use animated_emoji::{
    AnimatedEmoji, AnimatedEmojiWithIntensity, AnimationIntensity, EmojiType,
};
// TODO: [DEBT] Remove these re-exports after migrating all usages to contexts/ and hooks/
pub use animation_context::{
    use_animation_context, use_animation_context_or_default, AnimationContext,
};
pub use confirm_dialog::ConfirmDialog;
pub use dropdown_menu::{DropdownMenu, DropdownMenuDivider, DropdownMenuItem};
pub use features::auth::LoginCard;
pub use features::gamification::{
    BadgeGrid, ChallengeCard, ContributionGraph, ProfileCard, StatsDisplay, XpNotification,
};
pub use features::issues::{
    CreateIssueModal, CreateProjectModal, IssueCard, IssueClickEvent, IssueDetailModal,
    IssueDetailStatusChange, KanbanBoard, LinkRepositoryModal, StatusChangeEvent,
};
pub use features::tools::{LogViewer, ResultView, ToolDetail};
// TODO: [DEBT] Remove these re-exports after migrating all usages to contexts/ and hooks/
pub use network_status::{
    try_use_network_status, use_is_online, use_network_status, NetworkStatusProvider,
};
// OfflineBanner is now in ui::feedback, re-exported here for backward compatibility
pub use sidebar::Sidebar;
pub use ui::feedback::OfflineBanner;

// Page components - re-exported from pages module for backward compatibility
pub use pages::{
    HomePage, MockServerPage, ProjectDashboardPage, ProjectsPage, SettingsPage, XpHistoryPage,
};

// UI components (new paths)
pub use ui::{Card, CardVariant, Modal};
// Button components
pub use ui::button::{Button, ButtonSize, ButtonVariant, IconButton};
// Badge components
pub use ui::badge::{Badge, BadgeSize, BadgeVariant, DynamicBadge, Status, StatusBadge};
// Alert components
pub use ui::alert::{Alert, AlertVariant, Banner};
// Layout components
pub use ui::layout::{EmptyState, PageHeader, PageHeaderAction};
// Display components
pub use ui::display::{Avatar, AvatarSize, ProgressBar, ProgressBarVariant};
// Accordion components
pub use ui::accordion::{Accordion, AccordionItem, AccordionSection};
// Form components (new paths) - re-exported for backward compatibility
pub use ui::form::{LabeledToggle, OptionForm, ToggleSwitch, ToggleSwitchSize};
// Feedback components (new paths) - re-exported for backward compatibility
pub use ui::feedback::{InlineToast, SaveStatusIndicator, Toast, ToastType};
