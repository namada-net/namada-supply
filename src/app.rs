use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::http::{HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{BoxError, Json, Router};
use lazy_static::lazy_static;
use namada_sdk::tendermint_rpc::client::CompatMode;
use namada_sdk::tendermint_rpc::HttpClient;
use serde_json::json;
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::client::Client;
use crate::config::AppConfig;
use crate::handlers::{get_effective_supply, get_total_supply};
use crate::state::CommonState;

lazy_static! {
    static ref HTTP_TIMEOUT: u64 = 60;
    static ref REQ_PER_SEC: u64 = u64::MAX;
}

pub struct ApplicationServer;

impl ApplicationServer {
    pub async fn serve(config: AppConfig) -> anyhow::Result<()> {
        let client = HttpClient::builder(config.tendermint_url.as_str().parse().unwrap())
            .compat_mode(CompatMode::V0_37)
            .build()
            .unwrap();

        let routes = {
            let client = Client::new(client).await;
            let state = CommonState::new(client);
            Router::new()
                .route("/total-supply", get(get_total_supply))
                .route("/effective-supply", get(get_effective_supply))
                .with_state(state)
        };

        let cors = CorsLayer::new()
            .allow_origin("*".parse::<HeaderValue>().unwrap())
            .allow_methods(Any)
            .allow_headers(Any);

        let router = Router::new()
            .nest("/api/v1", routes)
            .merge(Router::new().route(
                "/health",
                get(|| async { json!({"commit": env!("VERGEN_GIT_SHA").to_string(), "version": env!("CARGO_PKG_VERSION") }).to_string() }),
            ))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(HandleErrorLayer::new(Self::handle_timeout_error))
                    .timeout(Duration::from_secs(*HTTP_TIMEOUT))
                    .layer(cors)
                    .layer(BufferLayer::new(4096))
                    .layer(RateLimitLayer::new(
                        *REQ_PER_SEC,
                        Duration::from_secs(1),
                    )),
            );

        let router = router.fallback(Self::handle_404);

        let port = config.port;
        let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));

        tracing::info!("🚀 Server has launched on https://{addr}");

        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

        axum::serve(listener, router.into_make_service())
            .with_graceful_shutdown(Self::shutdown_signal())
            .await
            .unwrap_or_else(|e| panic!("Server error: {}", e));

        Ok(())
    }

    /// Adds a custom handler for tower's `TimeoutLayer`, see https://docs.rs/axum/latest/axum/middleware/index.html#commonly-used-middleware.
    async fn handle_timeout_error(err: BoxError) -> (StatusCode, Json<serde_json::Value>) {
        if err.is::<tower::timeout::error::Elapsed>() {
            (
                StatusCode::REQUEST_TIMEOUT,
                Json(json!({
                    "error":
                        format!(
                            "request took longer than the configured {} second timeout",
                            *HTTP_TIMEOUT
                        )
                })),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("unhandled internal error: {}", err)
                })),
            )
        }
    }

    /// Tokio signal handler that will wait for a user to press CTRL+C.
    /// We use this in our hyper `Server` method `with_graceful_shutdown`.
    async fn shutdown_signal() {
        tokio::signal::ctrl_c()
            .await
            .expect("expect tokio signal ctrl-c");
        tracing::warn!("signal shutdown");
    }

    async fn handle_404() -> impl IntoResponse {
        (
            StatusCode::NOT_FOUND,
            axum::response::Json(serde_json::json!({
                    "errors":{
                    "message": vec!(String::from("The requested resource does not exist on this server!")),}
                }
            )),
        )
    }
}
