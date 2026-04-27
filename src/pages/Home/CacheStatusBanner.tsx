/**
 * CacheStatusBanner Component
 *
 * Inline banner shown on the Home dashboard when one of the SWR-style data
 * loaders falls back to cached data or fails to revalidate. Distinct from the
 * global `OfflineBanner` — that signals network state, this one signals the
 * freshness of the data the dashboard is currently rendering.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/182
 */

import { Icon } from '@/components/icons';

interface CacheStatusBannerProps {
  /** Whether any of the visible data is being served from the cache. */
  fromCache: boolean;
  /** Whether the latest revalidation attempt failed. */
  hasError: boolean;
  /** Whether a background revalidation is currently in flight. */
  isRevalidating: boolean;
  /** ISO8601 timestamp of the cached data being displayed. */
  cachedAt: string | null;
  /** Trigger a manual revalidation. */
  onRetry?: () => void;
}

function formatTimestamp(isoString: string | null): string | null {
  if (!isoString) return null;
  const date = new Date(isoString);
  if (Number.isNaN(date.getTime())) return null;
  const hours = date.getHours().toString().padStart(2, '0');
  const minutes = date.getMinutes().toString().padStart(2, '0');
  return `${hours}:${minutes}`;
}

export const CacheStatusBanner = ({
  fromCache,
  hasError,
  isRevalidating,
  cachedAt,
  onRetry,
}: CacheStatusBannerProps) => {
  if (!fromCache && !hasError) return null;

  const cachedTime = formatTimestamp(cachedAt);
  const message = hasError
    ? '最新化に失敗しました。前回取得したデータを表示中です。'
    : 'キャッシュされたデータを表示中です。';

  return (
    <div
      role="status"
      className="flex items-center gap-2 px-4 py-2 mb-4 bg-amber-500/10 border border-amber-500/30 text-amber-200 text-sm rounded-lg"
    >
      <Icon name="alert-triangle" className="w-4 h-4 flex-shrink-0" />
      <span className="flex-1">
        {message}
        {cachedTime && (
          <span className="ml-2 text-amber-300/80 text-xs">
            最終更新: {cachedTime}
          </span>
        )}
      </span>
      {onRetry && (
        <button
          type="button"
          onClick={onRetry}
          disabled={isRevalidating}
          className="flex items-center gap-1 px-2 py-1 text-xs rounded hover:bg-amber-500/20 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Icon
            name="refresh-cw"
            className={`w-3 h-3 ${isRevalidating ? 'animate-spin' : ''}`}
          />
          <span>{isRevalidating ? '更新中...' : '再試行'}</span>
        </button>
      )}
    </div>
  );
};
