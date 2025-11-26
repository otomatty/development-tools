/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.rs",
  ],
  theme: {
    extend: {
      colors: {
        // カラーパレット（要件定義書より）
        'dt-bg': '#0f172a',       // slate-900
        'dt-card': '#1e293b',     // slate-800
        'dt-text': '#f8fafc',     // slate-50
        'dt-text-sub': '#94a3b8', // slate-400
        'dt-accent': '#3b82f6',   // blue-500
        'dt-success': '#22c55e',  // green-500
        'dt-warning': '#eab308',  // yellow-500
        'dt-error': '#ef4444',    // red-500
        'dt-orange': '#f97316',   // orange-500
      },
      fontFamily: {
        'sans': ['JetBrains Mono', 'SF Pro Display', 'system-ui', 'sans-serif'],
        'mono': ['JetBrains Mono', 'SF Mono', 'Consolas', 'monospace'],
      },
    },
  },
  plugins: [],
}

