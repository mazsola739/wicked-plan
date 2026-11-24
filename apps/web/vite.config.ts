import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

// Relative base so the build works from any path: https://mazsola739.github.io/wicked-plan/ on GitHub Pages.
// The tunnel (bun run tunnel) serves the dev server on a trycloudflare.com address.
export default defineConfig({
  base: './',
  plugins: [react()],
  server: {
    port: 5173,
    allowedHosts: ['.trycloudflare.com'],
  },
});
