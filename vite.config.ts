import { defineConfig } from 'vite';
import solid from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solid()],
  server: {
    port: 1520,
    strictPort: true,
  },
  build: {
    target: 'esnext',
    outDir: 'dist',
  },
  resolve: {
    alias: {
      '@': '/src',
    },
  },
});

