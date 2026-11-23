import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

// The tunnel (bun run tunnel) serves the dev server on a trycloudflare.com address.
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    allowedHosts: ['.trycloudflare.com'],
  },
});
