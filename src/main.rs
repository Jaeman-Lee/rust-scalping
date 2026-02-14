mod config;
mod dashboard;
mod exchange;
mod indicators;
mod strategy;
mod telegram;
mod trading;
mod utils;

use crate::config::AppConfig;
use crate::dashboard::state::{DashboardEvent, EngineState};
use crate::exchange::client::BinanceClient;
use crate::exchange::websocket::run_kline_stream;
use crate::trading::engine::TradingEngine;
use crate::utils::logger::{init_tracing, TradeLogger};
use clap::Parser;
use std::sync::Arc;
use tokio::sync::{broadcast, watch, RwLock};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "scalping-bot", about = "Binance scalping trading bot")]
struct Cli {
    /// Path to config file
    #[arg(short, long, default_value = "config/default.toml")]
    config: String,

    /// Dry run mode (no actual orders)
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file (optional)
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    // Load config
    let config = AppConfig::load(&cli.config)?;

    // Init logging
    init_tracing(&config.logging.level);

    info!("Starting scalping bot...");
    info!("Config: {}", cli.config);
    info!("Symbol: {}", config.strategy.symbol);
    info!("Testnet: {}", config.exchange.testnet);

    if cli.dry_run {
        info!("DRY RUN MODE - no real orders will be placed");
    }

    // Load API keys
    let api_key = AppConfig::api_key()?;
    let secret_key = AppConfig::secret_key()?;

    // Create Binance client
    let client = BinanceClient::new(&config.exchange.base_url, api_key, secret_key);

    // Test connectivity
    match client.server_time().await {
        Ok(time) => info!("Connected to Binance. Server time: {}", time),
        Err(e) => {
            error!("Failed to connect to Binance: {}", e);
            anyhow::bail!("Cannot connect to Binance API");
        }
    }

    // Setup channels
    let (kline_tx, _) = broadcast::channel(256);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Create shared state and event sender for dashboard/telegram
    let shared_state = Arc::new(RwLock::new(EngineState::new(
        config.strategy.symbol.clone(),
    )));
    let (event_tx, _) = broadcast::channel::<DashboardEvent>(256);

    // Trade logger
    let trade_logger = TradeLogger::new(&config.logging.trade_log_path)?;

    // Trading engine
    let mut engine = TradingEngine::new(
        config.clone(),
        client,
        trade_logger,
        shared_state.clone(),
        event_tx.clone(),
    )?;

    // Warm up indicators with historical data
    engine.warm_up().await?;

    // Spawn dashboard server if enabled
    if config.dashboard.enabled {
        let dashboard_config = config.dashboard.clone();
        let dashboard_state = shared_state.clone();
        let dashboard_event_tx = event_tx.clone();
        let dashboard_shutdown_rx = shutdown_rx.clone();
        tokio::spawn(async move {
            if let Err(e) = dashboard::server::start_dashboard_server(
                dashboard_config,
                dashboard_state,
                dashboard_event_tx,
                dashboard_shutdown_rx,
            )
            .await
            {
                error!("Dashboard server error: {}", e);
            }
        });
    }

    // Spawn telegram bot if enabled
    if config.telegram.enabled {
        match (AppConfig::telegram_bot_token(), AppConfig::telegram_chat_id()) {
            (Ok(token), Ok(chat_id)) => {
                let tg_state = shared_state.clone();
                let tg_event_tx = event_tx.clone();
                let tg_config = config.clone();
                tokio::spawn(async move {
                    if let Err(e) = telegram::bot::run_telegram_bot(
                        token, chat_id, tg_state, tg_event_tx, tg_config,
                    )
                    .await
                    {
                        error!("Telegram bot error: {}", e);
                    }
                });
            }
            _ => {
                warn!("Telegram enabled but TELEGRAM_BOT_TOKEN or TELEGRAM_CHAT_ID not set");
            }
        }
    }

    // Spawn WebSocket stream
    let ws_config = config.clone();
    let ws_shutdown_rx = shutdown_rx.clone();
    let ws_kline_tx = kline_tx.clone();
    let ws_handle = tokio::spawn(async move {
        if let Err(e) = run_kline_stream(
            &ws_config.exchange.ws_url,
            &ws_config.strategy.symbol,
            &ws_config.strategy.interval,
            ws_kline_tx,
            ws_shutdown_rx,
        )
        .await
        {
            error!("WebSocket stream error: {}", e);
        }
    });

    // Subscribe to kline events for the engine
    let engine_kline_rx = kline_tx.subscribe();
    let engine_shutdown_rx = shutdown_rx.clone();

    // Spawn trading engine
    let engine_handle = tokio::spawn(async move {
        if let Err(e) = engine.run(engine_kline_rx, engine_shutdown_rx).await {
            error!("Trading engine error: {}", e);
        }
    });

    // Wait for Ctrl+C
    info!("Bot is running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received. Stopping gracefully...");

    // Signal shutdown
    let _ = shutdown_tx.send(true);

    // Wait for tasks to finish
    let _ = tokio::time::timeout(tokio::time::Duration::from_secs(10), async {
        let _ = ws_handle.await;
        let _ = engine_handle.await;
    })
    .await;

    info!("Bot stopped.");
    Ok(())
}
