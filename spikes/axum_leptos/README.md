# axum + leptos fullstack spike

A Rust-only fullstack web app: Axum on the server, [Leptos](https://leptos.dev) for the UI. The frontend is compiled by
`cargo-leptos`.

## Prerequisites

- Rust toolchain
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- `cargo-leptos`: `cargo install cargo-leptos`
- `wasm-bindgen-cli`: 
  `cargo install -f wasm-bindgen-cli --version 0.2.104`.


## Run it

```bash
cd spikes/axum_leptos
cargo leptos serve      # builds server (native) + client (WASM), then serves
```

Then open <http://127.0.0.1:3000>.

For an HMR-style dev loop with file watching, use:

```bash
cargo leptos watch
```

For a production build:

```bash
cargo leptos build --release
./target/release/axum_leptos_spike
```

## How it works

### One source file, two compilation targets

`src/lib.rs` is compiled twice:

1. **Server build** — `cargo build --features=ssr` produces the native
   `axum_leptos_spike` binary. The `#[server]` macro expands into a real
   Axum POST handler at `/api/echo`. Components render to HTML strings.
2. **Client build** — `cargo build --features=hydrate --target=wasm32-unknown-unknown`
   produces a `.wasm` module. The same `#[server]` macro expands the
   function body into a `fetch()` call that POSTs to `/api/echo`.
   Components hydrate the server-rendered DOM and attach event handlers.

`cargo-leptos` orchestrates both builds plus `wasm-bindgen-cli` (which
generates the JS shim that loads the WASM and calls `hydrate()`).
