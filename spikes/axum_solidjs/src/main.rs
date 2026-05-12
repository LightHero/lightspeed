use aide::axum::routing::post_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::openapi::OpenApi;
use aide::scalar::Scalar;
use aide::transform::TransformOpenApi;
use axum::{Extension, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct EchoRequest {
    /// The message to echo back to the caller.
    message: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct EchoResponse {
    /// The message the server received, echoed verbatim.
    echo: String,
}

async fn echo_handler(Json(req): Json<EchoRequest>) -> Json<EchoResponse> {
    tracing::info!(message = %req.message, "echo");
    Json(EchoResponse { echo: req.message })
}

async fn serve_openapi(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    Json((*api).clone())
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Echo Spike API (Node version)")
        .summary("Axum + AIDE OpenAPI demonstration. Frontend in TypeScript via Vite.")
}

fn frontend_dist() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("frontend").join("dist")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    aide::generate::on_error(|error| {
        tracing::error!(%error, "OpenAPI generation error");
    });
    aide::generate::extract_schemas(true);

    let mut api = OpenApi::default();

    let dist = frontend_dist();
    if !dist.exists() {
        tracing::warn!(
            ?dist,
            "frontend not built — run `cd frontend && npm install && npm run build`, \
             or use `npm run dev` for a hot-reloading dev server proxied to this backend"
        );
    }

    let app = ApiRouter::new()
        .api_route(
            "/api/echo",
            post_with(echo_handler, |op| {
                op.id("echo").description("Echo back the message sent in the request body.")
            }),
        )
        .route("/api/openapi.json", axum::routing::get(serve_openapi))
        .route("/api/docs", Scalar::new("/api/openapi.json").axum_route())
        .finish_api_with(&mut api, api_docs)
        .layer(Extension(Arc::new(api)))
        .layer(TraceLayer::new_for_http())
        // CORS so the Vite dev server (port 5173) and `openapi-typescript` CLI
        // can fetch the OpenAPI spec from anywhere during development.
        .layer(CorsLayer::permissive())
        .fallback_service(ServeDir::new(dist));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tracing::info!(%addr, "listening");
    println!();
    println!("  Backend:   http://{addr}/");
    println!("  API docs:  http://{addr}/api/docs");
    println!("  OpenAPI:   http://{addr}/api/openapi.json");
    println!();
    println!("  For the Vite dev server with hot reload:");
    println!("    cd frontend && npm install && npm run dev");
    println!("    → open http://localhost:5173/");
    println!();

    axum::serve(listener, app).await.unwrap();
}
