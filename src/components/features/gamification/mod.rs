//! Gamification Feature Components
//!
//! Components for the gamification and XP system, including user profiles,
//! badges, challenges, and progress tracking.

pub mod badge_grid;
pub mod challenge_card;
pub mod contribution_graph;
pub mod profile_card;
pub mod stats_display;
pub mod xp_history;
pub mod xp_notification;

pub use badge_grid::BadgeGrid;
pub use challenge_card::ChallengeCard;
pub use contribution_graph::ContributionGraph;
pub use profile_card::ProfileCard;
pub use stats_display::StatsDisplay;
pub use xp_history::XpHistoryPage;
pub use xp_notification::XpNotification;
