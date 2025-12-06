// Frontend type definitions
//
// This module defines the data structures used in the frontend application.
// Split into submodules for better maintainability.

// Re-export all types from submodules
export * from './auth';
export * from './challenge';
export * from './gamification';
export * from './issue';
export * from './mock-server';
export * from './network';
export * from './settings';
export * from './tool';

/// オプション値のマップ
export type OptionValues = Record<string, unknown>;

/// アプリのページ
export enum AppPage {
  Home = 'Home',
  Tools = 'Tools',
  Projects = 'Projects',
  MockServer = 'MockServer',
  Settings = 'Settings',
  XpHistory = 'XpHistory',
}

/// ProjectDetail page with project ID
export interface ProjectDetailPage {
  type: 'ProjectDetail';
  projectId: number;
}

/// Union type for all pages
export type AppPageType = AppPage | ProjectDetailPage;

/// Check if page is ProjectDetail
export function isProjectDetailPage(page: AppPageType): page is ProjectDetailPage {
  return (
    page !== null &&
    typeof page === 'object' &&
    'type' in page &&
    page.type === 'ProjectDetail'
  );
}
