use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::DashboardConfig;
use crate::dashboard::handlers::{self, AppState};
use crate::dashboard::{EventSender, SharedState};

pub async fn start_dashboard_server(
    config: DashboardConfig,
    shared_state: SharedState,
    event_tx: EventSender,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let app_state = AppState {
        shared_state,
        event_tx,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/status", axum::routing::get(handlers::get_status))
        .route("/api/trades", axum::routing::get(handlers::get_trades))
        .route(
            "/api/indicators",
            axum::routing::get(handlers::get_indicators),
        )
        .route("/api/balance", axum::routing::get(handlers::get_balance))
        .route("/api/ws", axum::routing::get(handlers::ws_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!("Dashboard server starting on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.changed().await;
        })
        .await?;

    info!("Dashboard server stopped");
    Ok(())
}
