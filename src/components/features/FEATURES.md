# Features Components Specification

## Overview

The `features/` directory organizes business-logic components into feature-specific modules, promoting clear separation of concerns and improved maintainability.

## Structure

```
src/components/features/
├── gamification/     # Gamification and XP system
├── auth/             # Authentication components
├── issues/           # Issue management and projects
├── tools/            # Development tools integration
└── mock_server/      # Mock server components
```

## Feature Modules

### 1. Gamification (`gamification/`)

**Purpose**: Components for the XP system, badges, challenges, and user progress tracking.

**Components**:

- `BadgeGrid` - Displays earned badges with progress info
- `ChallengeCard` - Shows active challenges with progress bars
- `ContributionGraph` - GitHub-style contribution calendar
- `ProfileCard` - User profile and statistics display
- `StatsDisplay` - Aggregated XP and level statistics
- `XpHistoryPage` - Full page view of XP acquisition history
- `XpNotification` - XP gain notifications

**Dependencies**:

- `src/tauri_api.rs` - API calls for badge and challenge data
- `src/types/gamification.rs` - Type definitions
- `src/components/ui/*` - UI primitives (Card, Badge, ProgressBar, etc.)
- `src/components/network_status.rs` - Online/offline status

### 2. Authentication (`auth/`)

**Purpose**: User authentication and login flows.

**Components**:

- `LoginCard` - GitHub OAuth login interface

**Dependencies**:

- `src/tauri_api.rs` - Authentication API
- `src/components/ui/*` - UI primitives

### 3. Issues Management (`issues/`)

**Purpose**: Issue tracking, project management, and kanban board functionality.

**Components**:

- `IssueCard` - Individual issue card with status dropdown
- `IssueDetailModal` - Detailed issue information and editing
- `CreateIssueModal` - Form to create new issues
- `CreateProjectModal` - Form to create new projects
- `KanbanBoard` - Linear-style kanban board
- `LinkRepositoryModal` - GitHub repository linking interface
- `ProjectsPage` - Main projects list page
- `ProjectDashboard` - Project-specific dashboard with kanban

**Dependencies**:

- `src/types/issue.rs` - Issue and project types
- `src/tauri_api.rs` - Issue management API
- `src/components/icons.rs` - Icon components
- `src/components/ui/*` - UI primitives

### 4. Tools (`tools/`)

**Purpose**: Development tools integration and result display.

**Components**:

- `ToolDetail` - Tool configuration and execution interface
- `LogViewer` - Log display and filtering
- `ResultView` - Tool execution results display

**Dependencies**:

- `src/tauri_api.rs` - Tool execution API
- `src/types/tool.rs` - Tool type definitions
- `src/components/ui/*` - UI primitives

### 5. Mock Server (`mock_server/`)

**Purpose**: Mock server configuration and management.

**Components**:

- `MockServerPage` - Main mock server configuration page

**Dependencies**:

- `src/tauri_api.rs` - Mock server API
- `src/types/mock_server.rs` - Mock server types
- `src/components/ui/*` - UI primitives

## UI Component Usage Guidelines

Each feature module should utilize the UI primitives from `src/components/ui/`:

- **Layout**: `PageHeader`, `EmptyState`, `Card`
- **Input**: `Input`, `ToggleSwitch`, `OptionForm`
- **Feedback**: `Toast`, `Loading`, `SaveStatusIndicator`
- **Display**: `Avatar`, `Badge`, `ProgressBar`
- **Dialog**: `Modal`, `ConfirmDialog`
- **Actions**: `Button`, `IconButton`, `DropdownMenu`

## Import Paths

### For using components in other parts of the application

```rust
// Direct imports from features
use crate::components::features::gamification::BadgeGrid;
use crate::components::features::auth::LoginCard;
use crate::components::features::issues::ProjectsPage;

// Or re-exported from components module
use crate::components::{BadgeGrid, LoginCard, ProjectsPage};
```

### Internal feature imports

Feature modules should use relative imports:

```rust
// Within gamification module
use crate::components::ui::Card;
use crate::tauri_api;
```

## Architecture Principles

1. **Feature Isolation**: Each feature module is self-contained
2. **UI Component First**: Utilize `ui/` primitives for consistency
3. **Backward Compatibility**: Old module paths re-export from `features/` during migration
4. **Clear Dependencies**: Feature modules only depend on:
   - UI primitives (`ui/`)
   - Type definitions (`types/`)
   - Utility modules (`tauri_api`, `network_status`, etc.)
   - **NOT** on other feature modules (use props to pass data)

## Migration Status

Phase 3 implementation status:

- ✅ Gamification components moved
- ✅ Auth components moved
- ✅ Issues components moved
- ✅ Tools components moved
- ✅ Mock Server components moved
- ⏳ Spec files creation (in progress)
- ⏳ Pages layer separation (planned for Phase 4)

## Future Improvements

1. **Spec Files**: Create detailed `.spec.md` for each feature module
2. **Pages Separation**: Move page-level components to `src/components/pages/`
3. **Component Extraction**: Extract smaller reusable components (e.g., `LevelBadge`, `XpProgressBar`)
4. **Test Coverage**: Add comprehensive unit tests for each component
5. **Storybook Integration**: Create component catalog for design system

## Related Issue

- GitHub Issue #116: Phase 3 - Feature Components Organization
- Phase 2 Completion: Issue #115

## Related Documentation

- Architecture: `docs/ARCHITECTURE.md`
- Development: `docs/DEVELOPMENT.md`
- Phase 3 Plan: `docs/03_plans/issue-management/`
