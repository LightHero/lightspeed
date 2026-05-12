import { defineConfig } from 'vite';
import solid from 'vite-plugin-solid';

// During `npm run dev`, the Vite server runs on port 5173 and proxies any
// request whose path starts with `/api` to the Axum backend on port 3000.
// In production, Axum serves the contents of `dist/` directly, so the same
// `/api/...` URLs work without proxying.
export default defineConfig({
  plugins: [solid()],
  server: {
    port: 5173,
    proxy: {
      '/api': 'http://localhost:3000',
    },
  },
  build: {
    target: 'es2022',
  },
});
