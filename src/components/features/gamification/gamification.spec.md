# Gamification Feature Components Specification

## Related Files

- Implementation: `src/components/features/gamification/`
- Parent Module: `src/components/features/mod.rs`

## Related Documentation

- Issue: GitHub Issue #116 (Phase 3 - Feature Components Organization)
- Plan: `docs/03_plans/`
- Architecture: `src/components/features/FEATURES.md`

## Requirements

### Responsibility

The Gamification feature module manages all components related to:

- User XP system and progression
- Badge acquisition and display
- Challenge tracking and completion
- Contribution graph (GitHub-style activity calendar)
- User profile and statistics display
- XP history and notifications

### State Management

- Components use Leptos signals for local state
- XP/badge data fetched via `tauri_api::` calls
- Cache management handled by backend

### Components Overview

#### BadgeGrid

**Purpose**: Display user's earned and near-completion badges

**Props**:

- None (fetches data internally)

**Features**:

- Shows earned badges with descriptions
- Displays progress for near-completion badges
- Modal detail view for each badge
- Responsive grid layout using UI primitives

**Dependencies**:

- `ui/dialog::Modal` - Badge detail modal
- `tauri_api::get_badges_with_progress()` - Data fetching
- `types::BadgeWithProgress` - Type definition

#### ChallengeCard

**Purpose**: Display active challenges with progress

**Props**:

- None (fetches data internally)

**Features**:

- Lists active challenges for the current user
- Shows progress bars for each challenge
- Displays completion percentage
- Handles errors and offline scenarios

**Dependencies**:

- `tauri_api::get_active_challenges()` - Data fetching
- `types::ChallengeInfo` - Type definition
- `network_status::use_is_online()` - Offline detection

#### ContributionGraph

**Purpose**: Display GitHub-style contribution calendar

**Props**:

- `github_stats: ReadSignal<Option<GitHubStats>>` - GitHub statistics

**Features**:

- Renders calendar grid for contribution display
- Shows daily code statistics on hover
- Supports code lines vs contribution toggle
- Manual and auto-sync functionality
- Rate limit information display

**Dependencies**:

- `tauri_api::get_code_stats_summary()` - Code statistics
- `tauri_api::sync_code_stats()` - Sync operation
- `types::CodeStatsResponse` - Type definition

#### ProfileCard

**Purpose**: Display user profile and key statistics

**Props**:

- None (fetches data internally)

**Features**:

- Shows user avatar and basic info
- Displays current level and XP progress
- Shows XP milestone information
- Responsive layout adapted to screen size

**Dependencies**:

- `tauri_api::get_user_profile()` - User data
- `ui/display::Avatar` - Avatar display
- `ui/display::ProgressBar` - XP progress bar

#### StatsDisplay

**Purpose**: Show aggregated XP and gamification statistics

**Props**:

- None (fetches data internally)

**Features**:

- Displays total XP, current level, streak count
- Shows badges earned count
- Displays challenges completed
- Uses skeleton loading for data fetching

**Dependencies**:

- `tauri_api::get_user_stats()` - Statistics data
- `ui/skeleton::Skeleton*` - Loading states

#### XpHistoryPage

**Purpose**: Full-page view of XP acquisition history

**Props**:

- `set_current_page: WriteSignal<AppPage>` - Page navigation signal

**Features**:

- Lists all XP transactions with action type and date
- Shows XP amounts and totals
- Filterable by action type (commit, PR, issue, etc.)
- Sortable by date or amount
- Pagination support

**Dependencies**:

- `tauri_api::get_xp_history()` - History data
- `types::XpHistoryEntry` - Type definition

#### XpNotification

**Purpose**: Display XP gain notifications

**Props**:

- `event: Option<XpGainedEvent>` - Event data
- `on_close: Callback` - Dismissal handler

**Features**:

- Shows XP gained with visual feedback
- Displays level up notifications
- Shows multiple badges earned notification
- Auto-dismissal after timeout
- Animated entrance/exit

**Dependencies**:

- `types::XpGainedEvent` - Event type
- `ui/feedback::Toast` - Toast notification base

## Test Cases

### TC-001: BadgeGrid - Initial Load

- **Given**: BadgeGrid component mounted
- **When**: Component initializes
- **Then**: Badges are fetched and displayed in grid

### TC-002: BadgeGrid - Badge Modal

- **Given**: Badges loaded in grid
- **When**: User clicks a badge
- **Then**: Modal shows badge details (name, description, date earned)

### TC-003: ChallengeCard - Online Mode

- **Given**: ChallengeCard component with online status
- **When**: Component initializes
- **Then**: Challenges are fetched and displayed

### TC-004: ChallengeCard - Offline Mode

- **Given**: ChallengeCard component with offline status
- **When**: Component initializes
- **Then**: Cached challenges displayed (if available) or offline message shown

### TC-005: ContributionGraph - Code Stats Display

- **Given**: ContributionGraph with GitHub stats
- **When**: User hovers over a date
- **Then**: Tooltip shows code additions/deletions for that date

### TC-006: ContributionGraph - Auto Sync

- **Given**: ContributionGraph with no cached stats
- **When**: Component initializes
- **Then**: Auto-sync triggered to fetch code statistics

### TC-007: ProfileCard - Display

- **Given**: ProfileCard component
- **When**: Component initializes
- **Then**: User avatar, level, and XP progress displayed

### TC-008: StatsDisplay - Statistics Display

- **Given**: StatsDisplay component
- **When**: Component initializes
- **Then**: Total XP, level, streak, badges, and challenges stats displayed

### TC-009: XpHistoryPage - History List

- **Given**: XpHistoryPage component
- **When**: Page initializes
- **Then**: XP transaction history is fetched and displayed in table/list

### TC-010: XpNotification - XP Gain

- **Given**: XpNotification with XP gain event
- **When**: Event fired
- **Then**: Toast notification shows XP gained, auto-dismisses after 5 seconds

### TC-011: XpNotification - Level Up

- **Given**: XpNotification with level up event
- **When**: Level up event fired
- **Then**: Level up modal shown with celebration animation

### TC-012: XpNotification - Badges Earned

- **Given**: XpNotification with multiple badges event
- **When**: Multiple badges earned event fired
- **Then**: Badges notification component shown with list of new badges

## UI Component Usage Examples

All components in this module use the following UI primitives:

```rust
// Cards and containers
use crate::components::ui::{Card, CardVariant};

// Progress and display
use crate::components::ui::{ProgressBar, ProgressBarVariant, Avatar, AvatarSize};

// Badges
use crate::components::ui::{Badge, BadgeVariant, BadgeSize};

// Modals
use crate::components::ui::{Modal, ModalSize, ModalHeader, ModalBody};

// Feedback
use crate::components::ui::{Toast, ToastType, Loading};

// Layout
use crate::components::ui::{PageHeader, EmptyState};
```

## Migration History

- **Date**: 2025-12-04
- **From**: `src/components/home/` and related modules
- **To**: `src/components/features/gamification/`
- **Status**: Completed (Phase 3a)
- **Tests**: All existing tests passing

## Future Enhancements

1. **Component Extraction**:

   - `LevelBadge` - Separate level display component
   - `XpProgressBar` - Custom progress bar for XP

2. **Performance**:

   - Virtual scrolling for XP history
   - Pagination optimization

3. **Offline Support**:

   - Better offline fallback UI
   - Sync reconciliation

4. **Analytics**:
   - Track component interaction events
   - Performance monitoring

## Related Synchronizations

- User context passing (from auth module to gamification)
- XP sync with backend
- Offline/online status updates
