mod backtest;
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
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::sync::{broadcast, watch, RwLock};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "scalping-bot", about = "Binance scalping trading bot")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Path to config file
    #[arg(short, long, default_value = "config/default.toml", global = true)]
    config: String,

    /// Dry run mode (no actual orders)
    #[arg(long, default_value_t = false, global = true)]
    dry_run: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run live trading (default if no subcommand)
    Trade,
    /// Run backtest on historical data
    Backtest {
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        start: String,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        end: String,
        /// Output CSV file path
        #[arg(long)]
        output: Option<String>,
        /// Fee rate in percent (e.g. 0.1 = 0.1%)
        #[arg(long, default_value_t = 0.1)]
        fee_rate: f64,
        /// Initial simulation balance in USDT
        #[arg(long, default_value_t = 10000.0)]
        initial_balance: f64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    let config = AppConfig::load(&cli.config)?;
    init_tracing(&config.logging.level);

    match cli.command {
        Some(Command::Backtest {
            start,
            end,
            output,
            fee_rate,
            initial_balance,
        }) => run_backtest(config, start, end, output, fee_rate, initial_balance).await,
        Some(Command::Trade) | None => run_trade(config, cli.dry_run).await,
    }
}

async fn run_backtest(
    config: AppConfig,
    start: String,
    end: String,
    output: Option<String>,
    fee_rate: f64,
    initial_balance: f64,
) -> anyhow::Result<()> {
    use crate::backtest::data::fetch_klines_paginated;
    use crate::backtest::engine::BacktestEngine;
    use chrono::NaiveDate;

    info!("Starting backtest mode");
    info!(
        "Symbol: {}, Interval: {}",
        config.strategy.symbol, config.strategy.interval
    );
    info!("Period: {} ~ {}", start, end);
    info!(
        "Fee rate: {}%, Initial balance: ${}",
        fee_rate, initial_balance
    );

    let start_date = NaiveDate::parse_from_str(&start, "%Y-%m-%d")
        .map_err(|e| anyhow::anyhow!("Invalid start date '{}': {}", start, e))?;
    let end_date = NaiveDate::parse_from_str(&end, "%Y-%m-%d")
        .map_err(|e| anyhow::anyhow!("Invalid end date '{}': {}", end, e))?;

    if start_date >= end_date {
        anyhow::bail!("Start date must be before end date");
    }

    let start_ms = start_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp_millis() as u64;
    let end_ms = end_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp_millis() as u64;

    // Create client for data fetching
    let api_key = AppConfig::api_key()?;
    let secret_key = AppConfig::secret_key()?;
    let client = BinanceClient::new(&config.exchange.base_url, api_key, secret_key);

    // Test connectivity
    match client.server_time().await {
        Ok(time) => info!("Connected to Binance. Server time: {}", time),
        Err(e) => {
            error!("Failed to connect to Binance: {}", e);
            anyhow::bail!("Cannot connect to Binance API");
        }
    }

    // Fetch historical data
    info!("Fetching historical data...");
    let klines = fetch_klines_paginated(
        &client,
        &config.strategy.symbol,
        &config.strategy.interval,
        start_ms,
        end_ms,
    )
    .await?;

    if klines.is_empty() {
        anyhow::bail!("No kline data fetched for the given period");
    }

    info!("Fetched {} candles. Running simulation...", klines.len());

    // Run backtest
    let fee_decimal = fee_rate / 100.0;
    let engine = BacktestEngine::new(config, fee_decimal, initial_balance);
    let result = engine.run(&klines)?;

    // Print results
    print!("{}", result);

    // Write CSV if requested
    if let Some(csv_path) = output {
        std::fs::write(&csv_path, result.to_csv())?;
        info!("Trade details written to {}", csv_path);
        println!("Trade details written to {}", csv_path);
    }

    Ok(())
}

async fn run_trade(config: AppConfig, dry_run: bool) -> anyhow::Result<()> {
    info!("Starting scalping bot...");
    info!("Symbol: {}", config.strategy.symbol);
    info!("Testnet: {}", config.exchange.testnet);

    if dry_run {
        info!("DRY RUN MODE - no real orders will be placed");
    }

    let api_key = AppConfig::api_key()?;
    let secret_key = AppConfig::secret_key()?;
    let client = BinanceClient::new(&config.exchange.base_url, api_key, secret_key);

    match client.server_time().await {
        Ok(time) => info!("Connected to Binance. Server time: {}", time),
        Err(e) => {
            error!("Failed to connect to Binance: {}", e);
            anyhow::bail!("Cannot connect to Binance API");
        }
    }

    let (kline_tx, _) = broadcast::channel(256);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let shared_state = Arc::new(RwLock::new(EngineState::new(
        config.strategy.symbol.clone(),
    )));
    let (event_tx, _) = broadcast::channel::<DashboardEvent>(256);

    let trade_logger = TradeLogger::new(&config.logging.trade_log_path)?;

    let mut engine = TradingEngine::new(
        config.clone(),
        client,
        trade_logger,
        shared_state.clone(),
        event_tx.clone(),
    )?;

    engine.warm_up().await?;

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

    if config.telegram.enabled {
        match (
            AppConfig::telegram_bot_token(),
            AppConfig::telegram_chat_id(),
        ) {
            (Ok(token), Ok(chat_id)) => {
                let tg_state = shared_state.clone();
                let tg_event_tx = event_tx.clone();
                let tg_config = config.clone();
                tokio::spawn(async move {
                    if let Err(e) = telegram::bot::run_telegram_bot(
                        token,
                        chat_id,
                        tg_state,
                        tg_event_tx,
                        tg_config,
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

    let engine_kline_rx = kline_tx.subscribe();
    let engine_shutdown_rx = shutdown_rx.clone();

    let engine_handle = tokio::spawn(async move {
        if let Err(e) = engine.run(engine_kline_rx, engine_shutdown_rx).await {
            error!("Trading engine error: {}", e);
        }
    });

    info!("Bot is running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received. Stopping gracefully...");

    let _ = shutdown_tx.send(true);

    let _ = tokio::time::timeout(tokio::time::Duration::from_secs(10), async {
        let _ = ws_handle.await;
        let _ = engine_handle.await;
    })
    .await;

    info!("Bot stopped.");
    Ok(())
}
