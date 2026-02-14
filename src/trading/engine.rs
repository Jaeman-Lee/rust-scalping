use chrono::Datelike;

use crate::config::AppConfig;
use crate::dashboard::state::{
    DashboardEvent, IndicatorSnapshot, PositionSnapshot, RiskSnapshot, TradeSnapshot,
};
use crate::dashboard::{EventSender, SharedState};
use crate::exchange::client::BinanceClient;
use crate::exchange::models::WsKlineEvent;
use crate::indicators::calculator::IndicatorCalculator;
use crate::strategy::scalping::ScalpingStrategy;
use crate::strategy::signals::Signal;
use crate::trading::orders::OrderManager;
use crate::trading::position::Position;
use crate::trading::risk::RiskManager;
use crate::utils::logger::{TradeLogger, TradeRecord};
use chrono::Utc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

pub struct TradingEngine {
    config: AppConfig,
    client: BinanceClient,
    strategy: ScalpingStrategy,
    calculator: IndicatorCalculator,
    risk_manager: RiskManager,
    position: Option<Position>,
    trade_logger: TradeLogger,
    last_reset_day: u32,
    shared_state: SharedState,
    event_tx: EventSender,
    total_wins: u32,
    total_losses: u32,
}

impl TradingEngine {
    pub fn new(
        config: AppConfig,
        client: BinanceClient,
        trade_logger: TradeLogger,
        shared_state: SharedState,
        event_tx: EventSender,
    ) -> anyhow::Result<Self> {
        let calculator = IndicatorCalculator::new(
            config.strategy.ema_short,
            config.strategy.ema_long,
            config.strategy.rsi_period,
            config.strategy.bollinger_period,
            config.strategy.bollinger_std,
        )?;

        let strategy = ScalpingStrategy::new(config.strategy.clone());
        let risk_manager = RiskManager::new(config.trading.clone(), 0.0);

        Ok(Self {
            config,
            client,
            strategy,
            calculator,
            risk_manager,
            position: None,
            trade_logger,
            last_reset_day: Utc::now().ordinal(),
            shared_state,
            event_tx,
            total_wins: 0,
            total_losses: 0,
        })
    }

    /// Load historical klines to warm up indicators
    pub async fn warm_up(&mut self) -> anyhow::Result<()> {
        info!("Warming up indicators with historical data...");
        let klines = self
            .client
            .get_klines(
                &self.config.strategy.symbol,
                &self.config.strategy.interval,
                100,
            )
            .await?;

        let prices: Vec<f64> = klines.iter().map(|k| k.close).collect();
        self.calculator.warm_up(&prices);
        info!("Warmed up with {} historical candles", prices.len());
        Ok(())
    }

    /// Main trading loop
    pub async fn run(
        &mut self,
        mut kline_rx: broadcast::Receiver<WsKlineEvent>,
        mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    ) -> anyhow::Result<()> {
        info!("Trading engine started for {}", self.config.strategy.symbol);

        // Fetch initial balance
        match self.client.account_info().await {
            Ok(info) => {
                for balance in &info.balances {
                    if balance.asset == "USDT" {
                        self.risk_manager.update_balance(balance.free_f64());
                        info!("Initial USDT balance: {}", balance.free);
                        break;
                    }
                }
            }
            Err(e) => warn!("Failed to fetch initial balance: {}", e),
        }

        loop {
            tokio::select! {
                event = kline_rx.recv() => {
                    match event {
                        Ok(kline_event) => {
                            self.process_kline(&kline_event).await;
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            warn!("Missed {} kline events", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            info!("Kline channel closed");
                            break;
                        }
                    }
                }
                _ = shutdown_rx.changed() => {
                    info!("Shutdown signal received in trading engine");
                    break;
                }
            }
        }

        // Graceful shutdown: close open position
        if let Some(ref position) = self.position {
            warn!(
                "Closing open position on shutdown: entry={:.2}, qty={:.6}",
                position.entry_price, position.quantity
            );
            let order_manager = OrderManager::new(&self.client, &self.config.strategy.symbol);
            if let Err(e) = order_manager.market_sell(position.quantity).await {
                error!("Failed to close position on shutdown: {}", e);
            }
            self.position = None;
        }

        // Update shared state on shutdown
        {
            let mut state = self.shared_state.write().await;
            state.is_running = false;
        }
        let _ = self.event_tx.send(DashboardEvent::EngineStatusChanged {
            is_running: false,
            is_paused: false,
        });

        info!(
            "Trading engine stopped. Daily stats: trades={}, pnl={:.4}",
            self.risk_manager.daily_trades(),
            self.risk_manager.daily_pnl()
        );

        Ok(())
    }

    async fn process_kline(&mut self, event: &WsKlineEvent) {
        let kline = &event.kline;

        // Only process on closed candles for signal generation
        if !kline.is_closed {
            return;
        }

        let price = kline.close_f64();
        if price <= 0.0 {
            return;
        }

        // Check for daily reset
        let today = Utc::now().ordinal();
        if today != self.last_reset_day {
            self.risk_manager.reset_daily();
            self.last_reset_day = today;
        }

        // Update indicators
        let indicators = self.calculator.update(price);

        // Check if paused
        let is_paused = {
            let state = self.shared_state.read().await;
            state.is_paused
        };

        // Generate signal
        let signal = self.strategy.evaluate(&indicators, self.position.as_ref());

        if !is_paused {
            match signal {
                Signal::Buy(ref info) => {
                    info!("BUY signal: {}", info.reason);
                    self.execute_buy(price).await;
                }
                Signal::Sell(ref info) => {
                    info!("SELL signal: {}", info.reason);
                    self.execute_sell(price).await;
                }
                Signal::Hold => {}
            }
        }

        // Update shared state after processing
        self.update_shared_state(price, &indicators).await;
    }

    async fn update_shared_state(
        &self,
        price: f64,
        indicators: &crate::indicators::calculator::IndicatorValues,
    ) {
        let indicator_snapshot = IndicatorSnapshot {
            ema_short: indicators.ema_short,
            ema_long: indicators.ema_long,
            rsi: indicators.rsi,
            bb_upper: indicators.bb_upper,
            bb_middle: indicators.bb_middle,
            bb_lower: indicators.bb_lower,
        };

        let position_snapshot = self.position.as_ref().map(|p| PositionSnapshot {
            entry_price: p.entry_price,
            quantity: p.quantity,
            entry_time: p.entry_time,
            unrealized_pnl: p.unrealized_pnl(price),
            unrealized_pnl_pct: p.unrealized_pnl_pct(price),
        });

        {
            let mut state = self.shared_state.write().await;
            state.current_price = price;
            state.indicators = Some(indicator_snapshot.clone());
            state.position = position_snapshot;
            state.risk = RiskSnapshot {
                daily_trades: self.risk_manager.daily_trades(),
                daily_pnl: self.risk_manager.daily_pnl(),
                consecutive_losses: self.risk_manager.consecutive_losses(),
                account_balance: self.risk_manager.account_balance(),
                max_daily_trades: self.risk_manager.max_daily_trades(),
                max_daily_loss_pct: self.risk_manager.max_daily_loss_pct(),
                total_wins: self.total_wins,
                total_losses: self.total_losses,
            };
            state.last_update = Utc::now();
        }

        // Broadcast price update event (ignore if no subscribers)
        let _ = self.event_tx.send(DashboardEvent::PriceUpdate {
            price,
            symbol: self.config.strategy.symbol.clone(),
            indicators: Some(indicator_snapshot),
        });
    }

    async fn execute_buy(&mut self, price: f64) {
        if !self.risk_manager.can_trade() {
            warn!("Risk manager blocked trade");
            let _ = self.event_tx.send(DashboardEvent::RiskAlert {
                message: "Risk manager blocked BUY trade".to_string(),
            });
            return;
        }

        let quantity = self.config.trading.quantity;
        if !self.risk_manager.check_position_size(quantity) {
            return;
        }

        let order_manager = OrderManager::new(&self.client, &self.config.strategy.symbol);
        match order_manager.market_buy(quantity).await {
            Ok(order) => {
                info!("BUY order filled: id={}", order.order_id);
                self.position = Some(Position::new(price, quantity, Utc::now()));

                let trade = TradeSnapshot {
                    side: "BUY".to_string(),
                    entry_price: price,
                    exit_price: 0.0,
                    quantity,
                    pnl: 0.0,
                    pnl_pct: 0.0,
                    timestamp: Utc::now(),
                };
                let _ = self.event_tx.send(DashboardEvent::TradeExecuted {
                    trade: trade.clone(),
                });
                {
                    let mut state = self.shared_state.write().await;
                    state.push_trade(trade);
                }
            }
            Err(e) => {
                error!("BUY order failed: {}", e);
            }
        }
    }

    async fn execute_sell(&mut self, price: f64) {
        let position = match self.position.take() {
            Some(p) => p,
            None => return,
        };

        let order_manager = OrderManager::new(&self.client, &self.config.strategy.symbol);
        match order_manager.market_sell(position.quantity).await {
            Ok(order) => {
                let pnl = position.unrealized_pnl(price);
                let pnl_pct = position.unrealized_pnl_pct(price);

                info!(
                    "SELL order filled: id={}, PnL={:.4} ({:.2}%)",
                    order.order_id, pnl, pnl_pct
                );

                self.risk_manager.record_trade(pnl);

                if pnl >= 0.0 {
                    self.total_wins += 1;
                } else {
                    self.total_losses += 1;
                }

                let trade = TradeSnapshot {
                    side: "SELL".to_string(),
                    entry_price: position.entry_price,
                    exit_price: price,
                    quantity: position.quantity,
                    pnl,
                    pnl_pct,
                    timestamp: Utc::now(),
                };

                let _ = self.event_tx.send(DashboardEvent::TradeExecuted {
                    trade: trade.clone(),
                });
                {
                    let mut state = self.shared_state.write().await;
                    state.push_trade(trade);
                }

                if let Err(e) = self.trade_logger.log_trade(&TradeRecord {
                    symbol: &self.config.strategy.symbol,
                    side: "SELL",
                    entry_price: position.entry_price,
                    exit_price: price,
                    quantity: position.quantity,
                    pnl,
                    pnl_pct,
                }) {
                    error!("Failed to log trade: {}", e);
                }
            }
            Err(e) => {
                error!("SELL order failed: {}", e);
                // Restore position if sell failed
                self.position = Some(position);
            }
        }
    }
}
