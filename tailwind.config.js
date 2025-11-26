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
        
        // ゲーミング風カラーパレット
        'gm-bg-primary': '#0D0D0D',
        'gm-bg-secondary': '#1A1A2E',
        'gm-bg-card': '#16213E',
        'gm-accent-cyan': '#00F5FF',
        'gm-accent-purple': '#BF00FF',
        'gm-accent-pink': '#FF00F5',
        'gm-success': '#00FF85',
        'gm-warning': '#FF9500',
        'gm-error': '#FF0055',
        
        // バッジレアリティカラー
        'badge-bronze': '#CD7F32',
        'badge-silver': '#C0C0C0',
        'badge-gold': '#FFD700',
        'badge-platinum': '#E5E4E2',
      },
      fontFamily: {
        'sans': ['JetBrains Mono', 'SF Pro Display', 'system-ui', 'sans-serif'],
        'mono': ['JetBrains Mono', 'SF Mono', 'Consolas', 'monospace'],
        'gaming': ['Orbitron', 'Rajdhani', 'system-ui', 'sans-serif'],
        'gaming-body': ['Rajdhani', 'system-ui', 'sans-serif'],
        'gaming-mono': ['Share Tech Mono', 'JetBrains Mono', 'monospace'],
      },
      boxShadow: {
        'neon-cyan': '0 0 5px #00F5FF, 0 0 20px #00F5FF40',
        'neon-purple': '0 0 5px #BF00FF, 0 0 20px #BF00FF40',
        'neon-pink': '0 0 5px #FF00F5, 0 0 20px #FF00F540',
        'neon-green': '0 0 5px #00FF85, 0 0 20px #00FF8540',
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'glow': 'glow 2s ease-in-out infinite alternate',
      },
      keyframes: {
        glow: {
          '0%': { boxShadow: '0 0 5px #00F5FF, 0 0 10px #00F5FF40' },
          '100%': { boxShadow: '0 0 10px #00F5FF, 0 0 30px #00F5FF60' },
        },
      },
    },
  },
  plugins: [],
}

