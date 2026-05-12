# axum + solidjs spike (Node.js + TypeScript)

## First-time setup

```bash
cd spikes/axum_solidjs_node
cargo build                                 # builds the backend
(cd frontend && npm install)                # installs Vite + Solid + openapi-typescript
```

## Hot reloading dev loop

Two terminals — backend on `:3000`, Vite on `:5173`. Vite proxies anything
under `/api` to Axum so the same code works in dev and in prod.

```bash
# terminal 1 — backend
cargo run

# terminal 2 — regenerate types from the live OpenAPI, then start Vite
cd frontend
npm run gen:api      # writes src/api/schema.ts from http://localhost:3000/api/openapi.json
npm run dev          # http://localhost:5173 with HMR
```

Open <http://localhost:5173>.


## Production build (example)

```bash
cd frontend
npm run gen:api      # refresh types
npm run build        # tsc --noEmit, then `vite build` → frontend/dist/
cd ..
cargo run            # http://localhost:3000 serves the built bundle
```

In prod mode the same `/api/...` URLs work because Axum hosts both the API
and the static bundle on port 3000 — no proxy needed.
