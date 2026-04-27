/**
 * Session Expired Banner
 *
 * Persistent banner shown across all routes whenever the backend has
 * detected that the stored GitHub token is no longer valid (revoked,
 * expired, or otherwise rejected). Mounted from `MainLayout` so it surfaces
 * regardless of which page the user is on when the 401 is observed.
 *
 * Behavior:
 * - Visible only while `useAuth.authExpired` is set.
 * - "再ログイン" routes the user to home where `LoginCard` lives.
 * - "閉じる" hides the banner for the current session; the next 401 will
 *   re-emit the event and re-show it.
 *
 * Related: src-tauri/src/auth/session.rs, Issue #181.
 */

import { useNavigate } from 'react-router-dom';
import { useAuth } from '../../../stores/authStore';
import { Icon } from '../../icons';

export const SessionExpiredBanner = () => {
  const authExpired = useAuth((s) => s.authExpired);
  const dismiss = useAuth((s) => s.dismissAuthExpired);
  const navigate = useNavigate();

  if (!authExpired) return null;

  const handleRelogin = () => {
    dismiss();
    // LoginCard lives on the home page; routing there guarantees the user
    // sees the device-flow CTA even if they were deep in another tab.
    navigate('/');
  };

  return (
    <div
      role="alert"
      aria-live="assertive"
      className="bg-red-500/90 text-red-50 px-4 py-2 text-sm flex items-center justify-center gap-3 flex-wrap"
    >
      <Icon name="alert-triangle" className="w-4 h-4 flex-shrink-0" />
      <span className="font-medium">{authExpired.message}</span>
      <button
        type="button"
        onClick={handleRelogin}
        className="px-3 py-1 rounded-md bg-white/20 hover:bg-white/30 active:bg-white/40 transition-colors text-white text-xs font-semibold"
      >
        再ログイン
      </button>
      <button
        type="button"
        onClick={dismiss}
        aria-label="閉じる"
        className="px-2 py-1 rounded-md hover:bg-white/10 active:bg-white/20 transition-colors text-white/80 hover:text-white text-xs"
      >
        閉じる
      </button>
    </div>
  );
};
