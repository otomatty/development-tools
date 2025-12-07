/**
 * Login Card Component
 *
 * Solid.js implementation of LoginCard component.
 * Displays when user is not logged in.
 * Supports GitHub Device Flow authentication.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ./login_card.rs
 */

import { Component, Show, createSignal, onCleanup } from 'solid-js';
import { useNetworkStatus } from '../../../stores/networkStore';
import { useAuth } from '../../../stores/authStore';
import { auth as authApi } from '../../../lib/tauri/commands';
import { Button } from '../../ui/button';
import type { DeviceCodeResponse, DeviceTokenStatus } from '../../../types';

type LoginState =
  | { type: 'Initial' }
  | { type: 'Starting' }
  | { type: 'WaitingForCode'; userCode: string; verificationUri: string; expiresIn: number }
  | { type: 'Polling' }
  | { type: 'Error'; message: string };

// Initial view with login button
const InitialView: Component<{ onLogin: () => void }> = (props) => {
  const network = useNetworkStatus();
  const isOnline = () => network.isOnline();

  return (
    <>
      {/* Title */}
      <h2 class="text-3xl font-gaming font-bold text-white mb-4">Level Up Your Dev Game</h2>

      {/* Description */}
      <p class="text-dt-text-sub mb-8 font-gaming-body text-lg">
        Track your GitHub activity, earn XP, unlock badges, and become a legendary developer.
      </p>

      {/* Features list */}
      <div class="text-left mb-8 space-y-3">
        <div class="flex items-center gap-3 text-dt-text-sub">
          <span class="text-gm-success">✓</span>
          <span>Track commits, PRs, and reviews</span>
        </div>
        <div class="flex items-center gap-3 text-dt-text-sub">
          <span class="text-gm-success">✓</span>
          <span>Earn XP and level up</span>
        </div>
        <div class="flex items-center gap-3 text-dt-text-sub">
          <span class="text-gm-success">✓</span>
          <span>Unlock achievement badges</span>
        </div>
        <div class="flex items-center gap-3 text-dt-text-sub">
          <span class="text-gm-success">✓</span>
          <span>Maintain your commit streak</span>
        </div>
      </div>

      {/* Login button - disabled when offline */}
      <div class="relative group">
        <Button
          variant="primary"
          onClick={props.onLogin}
          disabled={!isOnline()}
          fullWidth
          class={
            !isOnline()
              ? 'opacity-50 cursor-not-allowed'
              : 'shadow-neon-cyan'
          }
        >
          <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
          </svg>
          Connect with GitHub
        </Button>

        {/* Offline tooltip */}
        <Show when={!isOnline()}>
          <div class="absolute -bottom-10 left-1/2 -translate-x-1/2 px-3 py-1.5 bg-gm-bg-dark/95 text-gm-warning text-xs rounded-lg border border-gm-warning/30 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-10">
            ⚠️ オフラインのためログインできません
          </div>
        </Show>
      </div>

      {/* Note */}
      <p class="mt-6 text-xs text-dt-text-sub">
        We only request read access to your public activity.
      </p>
    </>
  );
};

// Loading view while starting device flow
const StartingView: Component = () => {
  return (
    <>
      <h2 class="text-2xl font-gaming font-bold text-white mb-4">Starting Authentication...</h2>
      <div class="flex justify-center mb-8">
        <div class="animate-spin rounded-full h-12 w-12 border-4 border-gm-accent-cyan border-t-transparent"></div>
      </div>
      <p class="text-dt-text-sub">Please wait while we set up the connection.</p>
    </>
  );
};

// View showing the device code for user to enter
const WaitingForCodeView: Component<{
  userCode: string;
  verificationUri: string;
  onCancel: () => void;
  onOpenUrl: (url: string) => void;
}> = (props) => {
  const [copied, setCopied] = createSignal(false);

  // Copy to clipboard function
  const copyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(props.userCode);
      setCopied(true);
      // Reset after 2 seconds
      setTimeout(() => {
        setCopied(false);
      }, 2000);
    } catch (e) {
      console.error('Failed to copy to clipboard:', e);
    }
  };

  return (
    <>
      <h2 class="text-2xl font-gaming font-bold text-white mb-4">Enter This Code on GitHub</h2>

      {/* User code display with copy button */}
      <div class="bg-gm-bg-dark/50 rounded-xl p-6 mb-6 border border-gm-accent-purple/30">
        <p class="text-sm text-dt-text-sub mb-2">Your code:</p>
        <div class="flex items-center justify-center gap-3">
          <div class="text-4xl font-mono font-bold text-gm-accent-cyan tracking-widest select-all">
            {props.userCode}
          </div>
          <button
            class="p-2 rounded-lg bg-gm-accent-cyan/20 hover:bg-gm-accent-cyan/30 transition-colors border border-gm-accent-cyan/30 group"
            onClick={copyToClipboard}
            title="Copy to clipboard"
          >
            <Show
              when={copied()}
              fallback={
                <svg class="w-6 h-6 text-gm-accent-cyan group-hover:text-white transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
              }
            >
              <svg class="w-6 h-6 text-gm-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </Show>
          </button>
        </div>
        {/* Copy feedback message */}
        <div class="h-6 mt-2">
          <Show when={copied()}>
            <p class="text-sm text-gm-success animate-fade-in">✓ Copied to clipboard!</p>
          </Show>
        </div>
      </div>

      {/* Instructions */}
      <div class="text-left mb-6 space-y-4">
        <div class="flex items-start gap-3">
          <span class="flex-shrink-0 w-6 h-6 bg-gm-accent-cyan/20 rounded-full flex items-center justify-center text-gm-accent-cyan text-sm font-bold">
            1
          </span>
          <span class="text-dt-text-sub">Click the button below to open GitHub</span>
        </div>
        <div class="flex items-start gap-3">
          <span class="flex-shrink-0 w-6 h-6 bg-gm-accent-cyan/20 rounded-full flex items-center justify-center text-gm-accent-cyan text-sm font-bold">
            2
          </span>
          <span class="text-dt-text-sub">Enter the code shown above</span>
        </div>
        <div class="flex items-start gap-3">
          <span class="flex-shrink-0 w-6 h-6 bg-gm-accent-cyan/20 rounded-full flex items-center justify-center text-gm-accent-cyan text-sm font-bold">
            3
          </span>
          <span class="text-dt-text-sub">Authorize the application</span>
        </div>
      </div>

      {/* Open GitHub button */}
      <Button
        variant="primary"
        onClick={() => props.onOpenUrl(props.verificationUri)}
        fullWidth
        class="mb-4 shadow-neon-cyan"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
        </svg>
        Open GitHub
      </Button>

      {/* Cancel button */}
      <Button variant="ghost" onClick={props.onCancel} fullWidth>
        Cancel
      </Button>

      {/* URL hint */}
      <p class="mt-4 text-xs text-dt-text-sub">
        Or visit: <span class="text-gm-accent-cyan">{props.verificationUri}</span>
      </p>
    </>
  );
};

// View showing polling status
const PollingView: Component<{ onCancel: () => void }> = (props) => {
  return (
    <>
      <h2 class="text-2xl font-gaming font-bold text-white mb-4">Waiting for Authorization...</h2>

      <div class="flex justify-center mb-6">
        <div class="animate-spin rounded-full h-12 w-12 border-4 border-gm-accent-purple border-t-transparent"></div>
      </div>

      <p class="text-dt-text-sub mb-6">
        Complete the authorization in your browser. This page will update automatically.
      </p>

      {/* Cancel button */}
      <Button variant="ghost" onClick={props.onCancel}>
        Cancel
      </Button>
    </>
  );
};

// Error view with retry option
const ErrorView: Component<{ message: string; onRetry: () => void }> = (props) => {
  const network = useNetworkStatus();
  const isOnline = () => network.isOnline();

  return (
    <>
      <div class="text-gm-error text-5xl mb-4">⚠️</div>

      <h2 class="text-2xl font-gaming font-bold text-white mb-4">Authentication Failed</h2>

      <p class="text-dt-text-sub mb-6">{props.message}</p>

      {/* Retry button - disabled when offline */}
      <div class="relative group">
        <Button
          variant="primary"
          onClick={props.onRetry}
          disabled={!isOnline()}
          fullWidth
          class={
            !isOnline()
              ? 'opacity-50 cursor-not-allowed'
              : 'shadow-neon-cyan'
          }
        >
          Try Again
        </Button>

        {/* Offline tooltip */}
        <Show when={!isOnline()}>
          <div class="absolute -bottom-10 left-1/2 -translate-x-1/2 px-3 py-1.5 bg-gm-bg-dark/95 text-gm-warning text-xs rounded-lg border border-gm-warning/30 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-10">
            ⚠️ オフラインのため再試行できません
          </div>
        </Show>
      </div>
    </>
  );
};

export const LoginCard: Component = () => {
  const auth = useAuth();
  const [loginState, setLoginState] = createSignal<LoginState>({ type: 'Initial' });
  const [pollingInterval, setPollingInterval] = createSignal<number | null>(null);

  // Handle login with Device Flow
  const onLogin = async () => {
    setLoginState({ type: 'Starting' });
    try {
      const deviceResponse: DeviceCodeResponse = await authApi.startDeviceFlow();
      setLoginState({
        type: 'WaitingForCode',
        userCode: deviceResponse.userCode,
        verificationUri: deviceResponse.verificationUri,
        expiresIn: deviceResponse.expiresIn,
      });
    } catch (e) {
      setLoginState({ type: 'Error', message: `Failed to start login: ${e}` });
    }
  };

  // Handle opening verification URL and start polling
  const onOpenUrl = async (url: string) => {
    try {
      await authApi.openUrl(url);
      setLoginState({ type: 'Polling' });

      // Start polling for token
      const interval = setInterval(async () => {
        try {
          const status: DeviceTokenStatus = await authApi.pollDeviceToken();
          if (status.status === 'success') {
            // Success - refresh auth state
            await auth.fetchAuthState();
            setLoginState({ type: 'Initial' });
            if (pollingInterval() !== null) {
              clearInterval(pollingInterval()!);
              setPollingInterval(null);
            }
          } else if (status.status === 'error') {
            setLoginState({ type: 'Error', message: status.message });
            if (pollingInterval() !== null) {
              clearInterval(pollingInterval()!);
              setPollingInterval(null);
            }
          }
          // If status is 'pending', continue polling
        } catch (e) {
          setLoginState({ type: 'Error', message: `Polling failed: ${e}` });
          if (pollingInterval() !== null) {
            clearInterval(pollingInterval()!);
            setPollingInterval(null);
          }
        }
      }, 5000); // Poll every 5 seconds

      setPollingInterval(interval);
    } catch (e) {
      setLoginState({ type: 'Error', message: `Failed to open URL: ${e}` });
    }
  };

  // Handle cancel
  const onCancel = async () => {
    if (pollingInterval() !== null) {
      clearInterval(pollingInterval()!);
      setPollingInterval(null);
    }
    try {
      await authApi.cancelDeviceFlow();
    } catch (e) {
      console.error('Failed to cancel device flow:', e);
    }
    setLoginState({ type: 'Initial' });
  };

  // Cleanup polling interval on unmount
  onCleanup(() => {
    if (pollingInterval() !== null) {
      clearInterval(pollingInterval()!);
    }
  });

  const state = () => loginState();

  return (
    <div class="flex items-center justify-center min-h-[60vh]">
      <div class="text-center p-12 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 shadow-neon-cyan max-w-md w-full">
        {/* Logo/Icon */}
        <div class="mb-8">
          <div class="w-24 h-24 mx-auto bg-gradient-to-br from-gm-accent-cyan to-gm-accent-purple rounded-2xl flex items-center justify-center shadow-neon-cyan">
            <span class="text-5xl">🎮</span>
          </div>
        </div>

        {/* Dynamic content based on login state */}
        <Show
          when={state().type === 'Initial'}
          fallback={
            <Show
              when={state().type === 'Starting'}
              fallback={
                <Show
                  when={state().type === 'WaitingForCode'}
                  fallback={
                    <Show
                      when={state().type === 'Polling'}
                      fallback={
                        state().type === 'Error' && (
                          <ErrorView message={state().message} onRetry={onLogin} />
                        )
                      }
                    >
                      <PollingView onCancel={onCancel} />
                    </Show>
                  }
                >
                  {state().type === 'WaitingForCode' && (
                    <WaitingForCodeView
                      userCode={state().userCode}
                      verificationUri={state().verificationUri}
                      onCancel={onCancel}
                      onOpenUrl={onOpenUrl}
                    />
                  )}
                </Show>
              }
            >
              <StartingView />
            </Show>
          }
        >
          <InitialView onLogin={onLogin} />
        </Show>
      </div>
    </div>
  );
};

