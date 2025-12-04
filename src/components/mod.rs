pub mod animated_emoji;
pub mod animation_context;
pub mod features;
pub mod home;
pub mod icons;
pub mod issues;
pub mod network_status;
pub mod settings;
pub mod sidebar;
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
pub use features::gamification::{BadgeGrid, ChallengeCard, ContributionGraph, ProfileCard, StatsDisplay, XpHistoryPage, XpNotification};
pub use features::auth::LoginCard;
pub use features::issues::{IssueCard, IssueDetailModal, CreateIssueModal, CreateProjectModal, KanbanBoard, LinkRepositoryModal, ProjectsPage, ProjectDashboard, IssueClickEvent, StatusChangeEvent, IssueDetailStatusChange};
pub use features::tools::{ToolDetail, LogViewer, ResultView};
pub use features::mock_server::MockServerPage;
pub use home::HomePage;
pub use network_status::{
    try_use_network_status, use_is_online, use_network_status, NetworkStatusProvider, OfflineBanner,
};
pub use sidebar::Sidebar;

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
