/**
 * Not Found Page
 *
 * 404 error page displayed when a route doesn't match any defined paths.
 *
 * Related Documentation:
 *   - Issue: https://github.com/otomatty/development-tools/issues/135
 */

import type { Component } from 'solid-js';

const NotFound: Component = () => {
  return (
    <div class="min-h-screen bg-dt-bg text-dt-text flex items-center justify-center">
      <div class="text-center">
        <h1 class="text-4xl font-bold text-gm-accent-cyan">404</h1>
        <p class="mt-4 text-dt-text-sub">Page Not Found</p>
      </div>
    </div>
  );
};

export default NotFound;

