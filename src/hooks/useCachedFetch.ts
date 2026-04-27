/**
 * useCachedFetch Hook
 *
 * SWR (Stale-While-Revalidate) style fetcher for Tauri commands that return
 * a `CachedResponse<T>` envelope. The hook exposes the cached data immediately
 * (via the backend cache fallback) and revalidates in the background based on
 * focus/reconnect events and a configurable `staleTime`.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/182
 *   - Backend: src-tauri/src/commands/github.rs (`get_*_with_cache`)
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { useNetworkStatus } from '@/stores/networkStore';
import type { CachedResponse } from '@/types';

export interface UseCachedFetchOptions {
  /** Disable fetching (e.g. while auth is unresolved). */
  enabled?: boolean;
  /** Milliseconds the data is considered fresh. Default 30 minutes. */
  staleTime?: number;
  /** Revalidate when the window regains focus. Default true. */
  revalidateOnFocus?: boolean;
  /** Revalidate when the network reconnects. Default true. */
  revalidateOnReconnect?: boolean;
}

export interface UseCachedFetchReturn<T> {
  /** Latest data (cached or fresh). */
  data: T | null;
  /** True while the first load is in flight. */
  isLoading: boolean;
  /** True while a background revalidation is in flight. */
  isRevalidating: boolean;
  /** The most recent error from the fetcher. */
  error: Error | null;
  /** Whether the currently displayed data came from the local cache. */
  fromCache: boolean;
  /** ISO8601 timestamp when the displayed data was cached. */
  cachedAt: string | null;
  /** ISO8601 timestamp when the cache expires. */
  expiresAt: string | null;
  /** Imperatively trigger a revalidation. */
  revalidate: () => Promise<void>;
}

const DEFAULT_STALE_TIME_MS = 30 * 60 * 1000;

/**
 * Wrap a Tauri command returning `CachedResponse<T>` with SWR-like semantics.
 *
 * The backend already implements cache-fallback, so the hook's role is to
 * surface the cache state to the UI and to schedule background revalidation
 * on focus/reconnect.
 */
export function useCachedFetch<T>(
  fetcher: () => Promise<CachedResponse<T>>,
  options: UseCachedFetchOptions = {},
): UseCachedFetchReturn<T> {
  const {
    enabled = true,
    staleTime = DEFAULT_STALE_TIME_MS,
    revalidateOnFocus = true,
    revalidateOnReconnect = true,
  } = options;

  const isOnline = useNetworkStatus(s => s.isOnline);

  const [data, setData] = useState<T | null>(null);
  const [fromCache, setFromCache] = useState(false);
  const [cachedAt, setCachedAt] = useState<string | null>(null);
  const [expiresAt, setExpiresAt] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(enabled);
  const [isRevalidating, setIsRevalidating] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const fetcherRef = useRef(fetcher);
  fetcherRef.current = fetcher;

  // Tracks when fresh (non-cache) data was last received. Used by the
  // focus-revalidation effect to gate against `staleTime` so we don't slam the
  // API every time the window is brought to the foreground.
  const lastFreshAtRef = useRef<number | null>(null);
  const inFlightRef = useRef(false);
  const mountedRef = useRef(true);
  // Mirror of `enabled` available inside the in-flight async closure. Lets us
  // discard a late response that arrived after the consumer disabled the hook
  // (e.g. the user logged out) so the previous user's data never leaks into
  // the next session.
  const enabledRef = useRef(enabled);
  enabledRef.current = enabled;
  // Mirror of `data` for use inside event handlers that should not re-bind
  // every render.
  const hasDataRef = useRef(false);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  const run = useCallback(async (mode: 'initial' | 'revalidate') => {
    if (!enabledRef.current) return;
    if (inFlightRef.current) return;
    inFlightRef.current = true;
    if (mode === 'initial') {
      setIsLoading(true);
    } else {
      setIsRevalidating(true);
    }
    try {
      const response = await fetcherRef.current();
      if (!mountedRef.current || !enabledRef.current) return;
      setData(response.data);
      hasDataRef.current = true;
      setFromCache(response.fromCache);
      setCachedAt(response.cachedAt);
      setExpiresAt(response.expiresAt);
      setError(null);
      // Only reset the freshness timer when the response actually came from
      // the API. If we returned cached data, the data is already considered
      // stale and a focus/reconnect should be free to revalidate immediately.
      if (!response.fromCache) {
        lastFreshAtRef.current = Date.now();
      }
    } catch (e) {
      if (!mountedRef.current || !enabledRef.current) return;
      setError(e instanceof Error ? e : new Error(String(e)));
    } finally {
      inFlightRef.current = false;
      if (mountedRef.current && enabledRef.current) {
        if (mode === 'initial') {
          setIsLoading(false);
        } else {
          setIsRevalidating(false);
        }
      }
    }
  }, []);

  // Initial load when enabled flips true; reset all state when it flips false
  // so a logout + login as a different user can't briefly show the previous
  // user's stats.
  useEffect(() => {
    if (!enabled) {
      setData(null);
      hasDataRef.current = false;
      setFromCache(false);
      setCachedAt(null);
      setExpiresAt(null);
      setError(null);
      setIsRevalidating(false);
      setIsLoading(false);
      lastFreshAtRef.current = null;
      return;
    }
    void run('initial');
  }, [enabled, run]);

  // Revalidate on focus once data is stale.
  useEffect(() => {
    if (!enabled || !revalidateOnFocus) return;
    const handler = () => {
      const last = lastFreshAtRef.current;
      if (last === null || Date.now() - last >= staleTime) {
        void run('revalidate');
      }
    };
    window.addEventListener('focus', handler);
    return () => window.removeEventListener('focus', handler);
  }, [enabled, revalidateOnFocus, run, staleTime]);

  // Revalidate when the network reconnects. We always trigger here — the
  // common case (came back online, want fresh data) and the recovery case
  // (initial load failed completely, have nothing on screen) both want a
  // re-fetch. `inFlightRef` dedupes against an in-progress initial load.
  useEffect(() => {
    if (!enabled || !revalidateOnReconnect) return;
    if (!isOnline) return;
    void run('revalidate');
  }, [enabled, isOnline, revalidateOnReconnect, run]);

  const revalidate = useCallback(async () => {
    await run('revalidate');
  }, [run]);

  return {
    data,
    isLoading,
    isRevalidating,
    error,
    fromCache,
    cachedAt,
    expiresAt,
    revalidate,
  };
}
