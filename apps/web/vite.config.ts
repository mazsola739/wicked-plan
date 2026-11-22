import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

// The Rust server (npm run server) listens on 3001; the page talks to it
// through /ws on the same origin, so the tunnel and phones on the Wi-Fi work too.
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    allowedHosts: ['.trycloudflare.com'],
    proxy: { '/ws': { target: 'ws://localhost:3001', ws: true } },
  },
});
