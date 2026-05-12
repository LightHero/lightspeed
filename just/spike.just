
# Builds the axum_leptos spike (release)
[group('spike')]
spike_axum_leptos_build:
  cd spikes/axum_leptos && cargo leptos build --release

# Starts the axum_leptos spike in dev mode on http://127.0.0.1:3000
[group('spike')]
spike_axum_leptos_start:
  cd spikes/axum_leptos && cargo leptos watch

# Builds the axum_solidjs spike (backend + frontend bundle)
[group('spike')]
spike_axum_solidjs_build:
  #!/usr/bin/env bash
  set -e
  cd spikes/axum_solidjs
  cargo build --release
  cd frontend
  npm install
  npm run build

# Starts the axum_solidjs backend on http://localhost:3000
[group('spike')]
spike_axum_solidjs_start:
  cd spikes/axum_solidjs && cargo run

# Starts the axum_solidjs Vite dev server on http://localhost:5173 (proxies /api to :3000)
[group('spike')]
spike_axum_solidjs_start_frontend:
  cd spikes/axum_solidjs/frontend && npm run dev
