use teloxide::prelude::*;
use tracing::info;

use crate::config::AppConfig;
use crate::dashboard::{EventSender, SharedState};
use crate::telegram::alerts::run_alert_listener;
use crate::telegram::commands::Command;

pub async fn run_telegram_bot(
    token: String,
    chat_id: i64,
    shared_state: SharedState,
    event_tx: EventSender,
    config: AppConfig,
) -> anyhow::Result<()> {
    info!("Starting Telegram bot...");

    let bot = Bot::new(token);

    // Spawn alert listener
    let alert_bot = bot.clone();
    let alert_rx = event_tx.subscribe();
    tokio::spawn(async move {
        run_alert_listener(alert_bot, chat_id, alert_rx).await;
    });

    // Setup command handler
    let handler = Update::filter_message().filter_command::<Command>().endpoint(
        move |bot: Bot, msg: Message, cmd: Command| {
            let state = shared_state.clone();
            let cfg = config.clone();
            async move {
                crate::telegram::commands::handle_command(bot, msg, cmd, state, cfg).await
            }
        },
    );

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    info!("Telegram bot stopped");
    Ok(())
}
