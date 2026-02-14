use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::{IntoResponse, Json};
use serde::Deserialize;
use tokio::sync::broadcast;

use crate::dashboard::state::DashboardEvent;
use crate::dashboard::SharedState;

#[derive(Clone)]
pub struct AppState {
    pub shared_state: SharedState,
    pub event_tx: broadcast::Sender<DashboardEvent>,
}

#[derive(Deserialize)]
pub struct TradesQuery {
    pub limit: Option<usize>,
}

pub async fn get_status(State(app): State<AppState>) -> impl IntoResponse {
    let state = app.shared_state.read().await;
    Json(serde_json::json!({
        "current_price": state.current_price,
        "symbol": state.symbol,
        "indicators": state.indicators,
        "position": state.position,
        "risk": state.risk,
        "is_running": state.is_running,
        "is_paused": state.is_paused,
        "last_update": state.last_update,
    }))
}

pub async fn get_trades(
    State(app): State<AppState>,
    Query(query): Query<TradesQuery>,
) -> impl IntoResponse {
    let state = app.shared_state.read().await;
    let limit = query.limit.unwrap_or(50).min(100);
    let trades: Vec<_> = state.recent_trades.iter().rev().take(limit).collect();
    Json(serde_json::json!({ "trades": trades }))
}

pub async fn get_indicators(State(app): State<AppState>) -> impl IntoResponse {
    let state = app.shared_state.read().await;
    Json(serde_json::json!({
        "indicators": state.indicators,
        "current_price": state.current_price,
    }))
}

pub async fn get_balance(State(app): State<AppState>) -> impl IntoResponse {
    let state = app.shared_state.read().await;
    let risk = &state.risk;
    Json(serde_json::json!({
        "account_balance": risk.account_balance,
        "daily_pnl": risk.daily_pnl,
        "daily_trades": risk.daily_trades,
        "win_rate": risk.win_rate(),
        "total_wins": risk.total_wins,
        "total_losses": risk.total_losses,
        "consecutive_losses": risk.consecutive_losses,
    }))
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(app): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, app))
}

async fn handle_ws(mut socket: WebSocket, app: AppState) {
    let mut rx = app.event_tx.subscribe();

    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(dashboard_event) => {
                        let json = match serde_json::to_string(&dashboard_event) {
                            Ok(j) => j,
                            Err(_) => continue,
                        };
                        if socket.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
