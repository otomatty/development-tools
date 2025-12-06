import type { Component } from 'solid-js';

const App: Component = () => {
  return (
    <div class="min-h-screen bg-dt-bg text-dt-text flex items-center justify-center">
      <div class="text-center space-y-4">
        <h1 class="text-4xl font-bold text-gm-accent-cyan">
          Hello Solid.js 🚀
        </h1>
        <p class="text-dt-text-sub">
          Development Tools - Solid.js Migration in Progress
        </p>
        <div class="flex items-center justify-center gap-4 mt-8">
          <a
            href="https://www.solidjs.com"
            target="_blank"
            rel="noopener noreferrer"
            class="px-4 py-2 bg-gm-accent-cyan/20 hover:bg-gm-accent-cyan/30 rounded-lg border border-gm-accent-cyan/50 transition-colors"
          >
            Solid.js Docs
          </a>
          <a
            href="https://v2.tauri.app"
            target="_blank"
            rel="noopener noreferrer"
            class="px-4 py-2 bg-gm-accent-purple/20 hover:bg-gm-accent-purple/30 rounded-lg border border-gm-accent-purple/50 transition-colors"
          >
            Tauri v2 Docs
          </a>
        </div>
      </div>
    </div>
  );
};

export default App;

