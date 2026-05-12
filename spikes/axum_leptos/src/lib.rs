use leptos::prelude::*;
use leptos_meta::{Link, MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

/// Top-level HTML shell rendered on the server. cargo-leptos injects the
/// hydration scripts and (in dev) the auto-reload listener.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        // Bundled stylesheet produced by cargo-leptos from style/main.css.
        <Stylesheet id="leptos" href="/pkg/axum_leptos_spike.css"/>
        // Pico CSS — classless framework loaded straight from a CDN.
        <Link
            rel="stylesheet"
            href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css"
        />
        <Title text="Echo spike — Axum + Leptos"/>

        <Router>
            <main class="container">
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=Home/>
                </Routes>
            </main>
        </Router>
    }
}

/// Server function. The `#[server]` macro generates two things:
///   * the `Echo` struct (request body for the call site).
///   * an Axum POST route at `/api/echo` registered automatically when the
///     `ssr` build wires up `generate_route_list(App)`.
/// On WASM the function body is replaced with a fetch to that route, so the
/// same call works from the client and the server.
#[server(Echo, prefix = "/api", endpoint = "echo")]
pub async fn echo_server(message: String) -> Result<String, ServerFnError> {
    Ok(message)
}

#[component]
fn Home() -> impl IntoView {
    let (input, set_input) = signal(String::from("hello from leptos"));

    let echo_action = ServerAction::<Echo>::new();
    let pending = echo_action.pending();
    let value = echo_action.value();

    view! {
        <article>
            <header>
                <hgroup>
                    <h1>"Echo spike"</h1>
                    <p>"Axum + Leptos fullstack, server-rendered then hydrated. No Node.js."</p>
                </hgroup>
            </header>

            <form on:submit=move |ev| {
                ev.prevent_default();
                echo_action.dispatch(Echo { message: input.get() });
            }>
                <label>
                    "Message"
                    <input
                        type="text"
                        prop:value=move || input.get()
                        on:input=move |ev| set_input.set(event_target_value(&ev))
                        disabled=move || pending.get()
                        placeholder="Type something and submit"
                        autofocus
                    />
                </label>
                <button type="submit" attr:aria-busy=move || pending.get()>
                    {move || if pending.get() { "Sending…" } else { "Send to /api/echo" }}
                </button>
            </form>

            {move || value.get().map(|result| match result {
                Ok(echoed) => view! {
                    <article>
                        <strong>"Server echoed: "</strong>
                        <code>{echoed}</code>
                    </article>
                }.into_any(),
                Err(err) => view! {
                    <article style="border-color: var(--pico-del-color, #c33);">
                        <strong>"Error: "</strong>
                        {err.to_string()}
                    </article>
                }.into_any(),
            })}
        </article>
    }
}

/// Entry point for the WASM bundle. Called by the JS shim that cargo-leptos
/// emits into `pkg/`.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
