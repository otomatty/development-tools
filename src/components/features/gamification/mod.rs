//! Gamification Feature Components
//!
//! Components for the gamification and XP system, including user profiles,
//! badges, challenges, and progress tracking.

pub mod badge_grid;
pub mod cache_indicator;
pub mod challenge_card;
pub mod contribution_graph;
pub mod dashboard_content;
pub mod home_data_loader;
pub mod profile_card;
pub mod stats_display;
pub mod sync_notifications;
pub mod xp_notification;

pub use badge_grid::BadgeGrid;
pub use cache_indicator::CacheIndicator;
pub use challenge_card::ChallengeCard;
pub use contribution_graph::ContributionGraph;
pub use dashboard_content::DashboardContent;
pub use home_data_loader::load_user_data;
pub use profile_card::ProfileCard;
pub use stats_display::StatsDisplay;
pub use sync_notifications::handle_sync_result_notifications;
pub use xp_notification::XpNotification;
