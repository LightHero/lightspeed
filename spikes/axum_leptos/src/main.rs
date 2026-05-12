#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use axum_leptos_spike::*;
    use leptos::config::get_configuration;
    use leptos_axum::{LeptosRoutes, generate_route_list};

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    // `generate_route_list` walks the `<Router>` tree in `App` *and* registers
    // every `#[server]` function as an Axum POST route under the configured
    // prefix (`/api/echo` in our case).
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!();
    println!("  Listening on http://{addr}/");
    println!("  Server function endpoint: POST http://{addr}/api/echo");
    println!();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // The browser-side build (hydrate feature) doesn't run `main`; it goes
    // through `lib::hydrate` via the JS shim emitted by cargo-leptos.
}
