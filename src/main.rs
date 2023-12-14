mod api;

use api::get_places;
use axum::{http::StatusCode, routing::get, Router};
use reqwest::Client;
use tower_http::trace::TraceLayer;

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn context() -> Client {
    reqwest::Client::new()
}

#[derive(Clone)]
pub struct AppState {
    client_reqwest: Client,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "multi_map_backend=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(fmt::layer())
        .init();

    let state = AppState {
        client_reqwest: context(),
    };
    let router = Router::new()
        .route("/health-check", get(|| async { (StatusCode::OK, "OK") }))
        .route("/places", get(get_places))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, router).await.unwrap();
}
