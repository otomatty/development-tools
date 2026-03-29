/**
 * Login Card Component
 *
 * React implementation of LoginCard component.
 * Displays when user is not logged in.
 * Supports GitHub Device Flow authentication.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/138
 *   - Original (Leptos): ./login_card.rs
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
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
const InitialView: React.FC<{ onLogin: () => void }> = ({ onLogin }) => {
  const isOnline = useNetworkStatus((s) => s.isOnline);

  return (
    <>
      {/* Title */}
      <h2 className="text-3xl font-gaming font-bold text-white mb-4">Level Up Your Dev Game</h2>

      {/* Description */}
      <p className="text-dt-text-sub mb-8 font-gaming-body text-lg">
        Track your GitHub activity, earn XP, unlock badges, and become a legendary developer.
      </p>

      {/* Features list */}
      <div className="text-left mb-8 space-y-3">
        <div className="flex items-center gap-3 text-dt-text-sub">
          <span className="text-gm-success">✓</span>
          <span>Track commits, PRs, and reviews</span>
        </div>
        <div className="flex items-center gap-3 text-dt-text-sub">
          <span className="text-gm-success">✓</span>
          <span>Earn XP and level up</span>
        </div>
        <div className="flex items-center gap-3 text-dt-text-sub">
          <span className="text-gm-success">✓</span>
          <span>Unlock achievement badges</span>
        </div>
        <div className="flex items-center gap-3 text-dt-text-sub">
          <span className="text-gm-success">✓</span>
          <span>Maintain your commit streak</span>
        </div>
      </div>

      {/* Login button - disabled when offline */}
      <div className="relative group">
        <Button
          variant="primary"
          onClick={onLogin}
          disabled={!isOnline}
          fullWidth
          className={
            !isOnline
              ? 'opacity-50 cursor-not-allowed'
              : 'shadow-neon-cyan'
          }
        >
          <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
          </svg>
          Connect with GitHub
        </Button>

        {/* Offline tooltip */}
        {!isOnline && (
          <div className="absolute -bottom-10 left-1/2 -translate-x-1/2 px-3 py-1.5 bg-gm-bg-dark/95 text-gm-warning text-xs rounded-lg border border-gm-warning/30 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-10">
            ⚠️ オフラインのためログインできません
          </div>
        )}
      </div>

      {/* Note */}
      <p className="mt-6 text-xs text-dt-text-sub">
        We only request read access to your public activity.
      </p>
    </>
  );
};

// Loading view while starting device flow
const StartingView: React.FC = () => {
  return (
    <>
      <h2 className="text-2xl font-gaming font-bold text-white mb-4">Starting Authentication...</h2>
      <div className="flex justify-center mb-8">
        <div className="animate-spin rounded-full h-12 w-12 border-4 border-gm-accent-cyan border-t-transparent"></div>
      </div>
      <p className="text-dt-text-sub">Please wait while we set up the connection.</p>
    </>
  );
};

// View showing the device code for user to enter
const WaitingForCodeView: React.FC<{
  userCode: string;
  verificationUri: string;
  onCancel: () => void;
  onOpenUrl: (url: string) => void;
}> = ({ userCode, verificationUri, onCancel, onOpenUrl }) => {
  const [copied, setCopied] = useState(false);
  const copyTimeoutRef = useRef<number | null>(null);

  // Cleanup copy timeout on unmount
  useEffect(() => {
    return () => {
      if (copyTimeoutRef.current !== null) {
        clearTimeout(copyTimeoutRef.current);
      }
    };
  }, []);

  // Copy to clipboard function
  const copyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(userCode);
      setCopied(true);
      // Clear any existing timeout
      if (copyTimeoutRef.current !== null) {
        clearTimeout(copyTimeoutRef.current);
      }
      // Reset after 2 seconds
      copyTimeoutRef.current = window.setTimeout(() => {
        setCopied(false);
        copyTimeoutRef.current = null;
      }, 2000);
    } catch (e) {
      console.error('Failed to copy to clipboard:', e);
    }
  };

  return (
    <>
      <h2 className="text-2xl font-gaming font-bold text-white mb-4">Enter This Code on GitHub</h2>

      {/* User code display with copy button */}
      <div className="bg-gm-bg-dark/50 rounded-xl p-6 mb-6 border border-gm-accent-purple/30">
        <p className="text-sm text-dt-text-sub mb-2">Your code:</p>
        <div className="flex items-center justify-center gap-3">
          <div className="text-4xl font-mono font-bold text-gm-accent-cyan tracking-widest select-all">
            {userCode}
          </div>
          <button
            className="p-2 rounded-lg bg-gm-accent-cyan/20 hover:bg-gm-accent-cyan/30 transition-colors border border-gm-accent-cyan/30 group"
            onClick={copyToClipboard}
            title="Copy to clipboard"
          >
            {copied ? (
              <svg className="w-6 h-6 text-gm-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7" />
              </svg>
            ) : (
              <svg className="w-6 h-6 text-gm-accent-cyan group-hover:text-white transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
            )}
          </button>
        </div>
        {/* Copy feedback message */}
        <div className="h-6 mt-2">
          {copied && (
            <p className="text-sm text-gm-success animate-fade-in">✓ Copied to clipboard!</p>
          )}
        </div>
      </div>

      {/* Instructions */}
      <div className="text-left mb-6 space-y-4">
        <div className="flex items-start gap-3">
          <span className="flex-shrink-0 w-6 h-6 bg-gm-accent-cyan/20 rounded-full flex items-center justify-center text-gm-accent-cyan text-sm font-bold">
            1
          </span>
          <span className="text-dt-text-sub">Click the button below to open GitHub</span>
        </div>
        <div className="flex items-start gap-3">
          <span className="flex-shrink-0 w-6 h-6 bg-gm-accent-cyan/20 rounded-full flex items-center justify-center text-gm-accent-cyan text-sm font-bold">
            2
          </span>
          <span className="text-dt-text-sub">Enter the code shown above</span>
        </div>
        <div className="flex items-start gap-3">
          <span className="flex-shrink-0 w-6 h-6 bg-gm-accent-cyan/20 rounded-full flex items-center justify-center text-gm-accent-cyan text-sm font-bold">
            3
          </span>
          <span className="text-dt-text-sub">Authorize the application</span>
        </div>
      </div>

      {/* Open GitHub button */}
      <Button
        variant="primary"
        onClick={() => onOpenUrl(verificationUri)}
        fullWidth
        className="mb-4 shadow-neon-cyan"
      >
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
        </svg>
        Open GitHub
      </Button>

      {/* Cancel button */}
      <Button variant="ghost" onClick={onCancel} fullWidth>
        Cancel
      </Button>

      {/* URL hint */}
      <p className="mt-4 text-xs text-dt-text-sub">
        Or visit: <span className="text-gm-accent-cyan">{verificationUri}</span>
      </p>
    </>
  );
};

// View showing polling status
const PollingView: React.FC<{ onCancel: () => void }> = ({ onCancel }) => {
  return (
    <>
      <h2 className="text-2xl font-gaming font-bold text-white mb-4">Waiting for Authorization...</h2>

      <div className="flex justify-center mb-6">
        <div className="animate-spin rounded-full h-12 w-12 border-4 border-gm-accent-purple border-t-transparent"></div>
      </div>

      <p className="text-dt-text-sub mb-6">
        Complete the authorization in your browser. This page will update automatically.
      </p>

      {/* Cancel button */}
      <Button variant="ghost" onClick={onCancel}>
        Cancel
      </Button>
    </>
  );
};

// Error view with retry option
const ErrorView: React.FC<{ message: string; onRetry: () => void }> = ({ message, onRetry }) => {
  const isOnline = useNetworkStatus((s) => s.isOnline);

  return (
    <>
      <div className="text-gm-error text-5xl mb-4">⚠️</div>

      <h2 className="text-2xl font-gaming font-bold text-white mb-4">Authentication Failed</h2>

      <p className="text-dt-text-sub mb-6">{message}</p>

      {/* Retry button - disabled when offline */}
      <div className="relative group">
        <Button
          variant="primary"
          onClick={onRetry}
          disabled={!isOnline}
          fullWidth
          className={
            !isOnline
              ? 'opacity-50 cursor-not-allowed'
              : 'shadow-neon-cyan'
          }
        >
          Try Again
        </Button>

        {/* Offline tooltip */}
        {!isOnline && (
          <div className="absolute -bottom-10 left-1/2 -translate-x-1/2 px-3 py-1.5 bg-gm-bg-dark/95 text-gm-warning text-xs rounded-lg border border-gm-warning/30 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-10">
            ⚠️ オフラインのため再試行できません
          </div>
        )}
      </div>
    </>
  );
};

export const LoginCard: React.FC = () => {
  const fetchAuthState = useAuth((s) => s.fetchAuthState);
  const [loginState, setLoginState] = useState<LoginState>({ type: 'Initial' });
  const pollingIntervalRef = useRef<number | null>(null);

  // Helper function to stop polling
  const stopPolling = useCallback(() => {
    if (pollingIntervalRef.current !== null) {
      clearInterval(pollingIntervalRef.current);
      pollingIntervalRef.current = null;
    }
  }, []);

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
      const interval = window.setInterval(async () => {
        try {
          const status: DeviceTokenStatus = await authApi.pollDeviceToken();
          if (status.status === 'success') {
            // Success - refresh auth state
            await fetchAuthState();
            setLoginState({ type: 'Initial' });
            clearInterval(interval);
            pollingIntervalRef.current = null;
          } else if (status.status === 'error') {
            setLoginState({ type: 'Error', message: status.message });
            clearInterval(interval);
            pollingIntervalRef.current = null;
          }
          // If status is 'pending', continue polling
        } catch (e) {
          setLoginState({ type: 'Error', message: `Polling failed: ${e}` });
          clearInterval(interval);
          pollingIntervalRef.current = null;
        }
      }, 5000); // Poll every 5 seconds

      pollingIntervalRef.current = interval;
    } catch (e) {
      setLoginState({ type: 'Error', message: `Failed to open URL: ${e}` });
    }
  };

  // Handle cancel
  const onCancel = async () => {
    stopPolling();
    try {
      await authApi.cancelDeviceFlow();
    } catch (e) {
      console.error('Failed to cancel device flow:', e);
    }
    setLoginState({ type: 'Initial' });
  };

  // Cleanup polling interval on unmount
  useEffect(() => {
    return () => {
      if (pollingIntervalRef.current !== null) {
        clearInterval(pollingIntervalRef.current);
      }
    };
  }, []);

  const renderContent = () => {
    switch (loginState.type) {
      case 'Initial':
        return <InitialView onLogin={onLogin} />;
      case 'Starting':
        return <StartingView />;
      case 'WaitingForCode':
        return (
          <WaitingForCodeView
            userCode={loginState.userCode}
            verificationUri={loginState.verificationUri}
            onCancel={onCancel}
            onOpenUrl={onOpenUrl}
          />
        );
      case 'Polling':
        return <PollingView onCancel={onCancel} />;
      case 'Error':
        return <ErrorView message={loginState.message} onRetry={onLogin} />;
      default:
        return null;
    }
  };

  return (
    <div className="flex items-center justify-center min-h-[60vh]">
      <div className="text-center p-12 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 shadow-neon-cyan max-w-md w-full">
        {/* Logo/Icon */}
        <div className="mb-8">
          <div className="w-24 h-24 mx-auto bg-gradient-to-br from-gm-accent-cyan to-gm-accent-purple rounded-2xl flex items-center justify-center shadow-neon-cyan">
            <span className="text-5xl">🎮</span>
          </div>
        </div>

        {/* Dynamic content based on login state */}
        {renderContent()}
      </div>
    </div>
  );
};
