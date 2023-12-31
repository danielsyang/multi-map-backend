mod api;

use std::env;

use api::{get_places, get_routes};
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use reqwest::Client;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn context() -> Client {
    reqwest::Client::new()
}

#[derive(Clone)]
pub struct AppState {
    client_reqwest: Client,
    google_key: String,
}

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let google_key = env::var("GOOGLE_PLACES_KEY").expect(".env file not found");

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
        google_key,
    };
    let router = Router::new()
        .route("/health-check", get(|| async { (StatusCode::OK, "OK") }))
        .route("/places", post(get_places))
        .route("/routes", post(get_routes))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, router).await.unwrap();
}
