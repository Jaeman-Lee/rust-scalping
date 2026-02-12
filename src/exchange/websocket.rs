use crate::exchange::models::WsKlineEvent;
use futures_util::StreamExt;
use tokio::sync::broadcast;
use tokio_tungstenite::connect_async;
use tracing::{error, info, warn};

pub async fn run_kline_stream(
    ws_url: &str,
    symbol: &str,
    interval: &str,
    tx: broadcast::Sender<WsKlineEvent>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let stream_name = format!("{}@kline_{}", symbol.to_lowercase(), interval);
    let url = format!("{}/{}", ws_url, stream_name);
    info!("Connecting to WebSocket: {}", url);

    loop {
        let connect_result = connect_async(&url).await;
        let (ws_stream, _) = match connect_result {
            Ok(conn) => {
                info!("WebSocket connected");
                conn
            }
            Err(e) => {
                error!("WebSocket connection failed: {}. Retrying in 5s...", e);
                tokio::select! {
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => continue,
                    _ = shutdown_rx.changed() => {
                        info!("Shutdown signal received, stopping WebSocket");
                        return Ok(());
                    }
                }
            }
        };

        let (_, mut read) = ws_stream.split();

        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(message)) => {
                            if let tokio_tungstenite::tungstenite::Message::Text(text) = message {
                                match serde_json::from_str::<WsKlineEvent>(&text) {
                                    Ok(event) => {
                                        let _ = tx.send(event);
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse kline event: {}", e);
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}. Reconnecting...", e);
                            break;
                        }
                        None => {
                            warn!("WebSocket stream ended. Reconnecting...");
                            break;
                        }
                    }
                }
                _ = shutdown_rx.changed() => {
                    info!("Shutdown signal received, stopping WebSocket");
                    return Ok(());
                }
            }
        }

        // Reconnect delay
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
