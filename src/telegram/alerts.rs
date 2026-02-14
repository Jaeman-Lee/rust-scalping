use teloxide::prelude::*;
use teloxide::types::ChatId;
use tokio::sync::broadcast;
use tracing::{error, warn};

use crate::dashboard::state::DashboardEvent;

pub async fn run_alert_listener(
    bot: Bot,
    chat_id: i64,
    mut event_rx: broadcast::Receiver<DashboardEvent>,
) {
    let chat = ChatId(chat_id);

    loop {
        match event_rx.recv().await {
            Ok(event) => {
                let text = match &event {
                    DashboardEvent::TradeExecuted { trade } => {
                        if trade.side == "BUY" {
                            format!(
                                "BUY {:.6} @ {:.2}",
                                trade.quantity, trade.entry_price,
                            )
                        } else {
                            let emoji = if trade.pnl >= 0.0 { "+" } else { "" };
                            format!(
                                "SELL {:.6} @ {:.2}\nPnL: {}{:.4} ({}{:.2}%)",
                                trade.quantity,
                                trade.exit_price,
                                emoji,
                                trade.pnl,
                                emoji,
                                trade.pnl_pct,
                            )
                        }
                    }
                    DashboardEvent::RiskAlert { message } => {
                        format!("Risk Alert: {}", message)
                    }
                    DashboardEvent::EngineStatusChanged {
                        is_running,
                        is_paused,
                    } => {
                        format!(
                            "Engine Status Changed\nRunning: {} | Paused: {}",
                            is_running, is_paused,
                        )
                    }
                    DashboardEvent::PriceUpdate { .. } => continue, // Skip price updates
                };

                if let Err(e) = bot.send_message(chat, &text).await {
                    error!("Failed to send Telegram alert: {}", e);
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                warn!("Telegram alert listener lagged by {} events", n);
            }
            Err(broadcast::error::RecvError::Closed) => {
                break;
            }
        }
    }
}
