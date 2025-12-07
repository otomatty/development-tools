/**
 * Settings Page
 *
 * Application settings page with accordion-style sections.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ../components/pages/settings/mod.rs
 */

import { Component, createSignal } from 'solid-js';
import { AccordionSection } from '../../components/ui/accordion';
import { Icon } from '../../components/icons';
import {
  AccountSettings,
  NotificationSettings,
  SyncSettings,
  AppearanceSettings,
  DataManagement,
  AppInfo,
  SettingsReset,
} from '../../components/features/settings';

type SettingsSection = 'Account' | 'Notification' | 'Sync' | 'Appearance' | 'DataManagement' | 'AppInfo';

export const Settings: Component = () => {
  const [expandedSections, setExpandedSections] = createSignal<Set<SettingsSection>>(
    new Set(['Account'])
  );

  const toggleSection = (section: SettingsSection) => {
    setExpandedSections((prev) => {
      const next = new Set(prev);
      if (next.has(section)) {
        next.delete(section);
      } else {
        next.add(section);
      }
      return next;
    });
  };

  const isExpanded = (section: SettingsSection) => expandedSections().has(section);

  return (
    <div class="flex-1 overflow-y-auto p-6">
      <h1 class="text-3xl font-gaming font-bold text-white mb-6 flex items-center gap-3">
        <Icon name="settings" class="w-8 h-8 text-gm-accent-cyan" />
        SETTINGS
      </h1>

      <div class="space-y-4">
        {/* Account Settings Section */}
        <AccordionSection
          title="アカウント設定"
          icon="user"
          expanded={() => isExpanded('Account')}
          onToggle={() => toggleSection('Account')}
          maxHeight="1000px"
        >
          <AccountSettings />
        </AccordionSection>

        {/* Notification Settings Section */}
        <AccordionSection
          title="通知設定"
          icon="bell"
          expanded={() => isExpanded('Notification')}
          onToggle={() => toggleSection('Notification')}
          maxHeight="1000px"
        >
          <NotificationSettings />
        </AccordionSection>

        {/* Sync Settings Section */}
        <AccordionSection
          title="同期設定"
          icon="refresh-cw"
          expanded={() => isExpanded('Sync')}
          onToggle={() => toggleSection('Sync')}
          maxHeight="1000px"
        >
          <SyncSettings />
        </AccordionSection>

        {/* Appearance Settings Section */}
        <AccordionSection
          title="外観設定"
          icon="palette"
          expanded={() => isExpanded('Appearance')}
          onToggle={() => toggleSection('Appearance')}
          maxHeight="500px"
        >
          <AppearanceSettings />
        </AccordionSection>

        {/* Data Management Section */}
        <AccordionSection
          title="データ管理"
          icon="database"
          expanded={() => isExpanded('DataManagement')}
          onToggle={() => toggleSection('DataManagement')}
          maxHeight="1200px"
        >
          <DataManagement />
        </AccordionSection>

        {/* App Info Section */}
        <AccordionSection
          title="アプリ情報"
          icon="info"
          expanded={() => isExpanded('AppInfo')}
          onToggle={() => toggleSection('AppInfo')}
          maxHeight="600px"
        >
          <AppInfo />
        </AccordionSection>

        {/* Settings Reset Section (not in accordion) */}
        <div class="mt-6">
          <SettingsReset />
        </div>
      </div>
    </div>
  );
};

export default Settings;
