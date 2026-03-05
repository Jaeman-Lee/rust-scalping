use crate::backtest::metrics::{BacktestResult, BacktestTrade};
use crate::config::AppConfig;
use crate::exchange::models::Kline;
use crate::indicators::calculator::IndicatorCalculator;
use crate::strategy::scalping::ScalpingStrategy;
use crate::strategy::signals::Signal;
use crate::trading::position::Position;
use crate::trading::risk::RiskManager;
use chrono::{DateTime, Datelike, TimeZone, Utc};
use tracing::info;

pub struct BacktestEngine {
    config: AppConfig,
    fee_rate: f64,
    initial_balance: f64,
}

impl BacktestEngine {
    pub fn new(config: AppConfig, fee_rate: f64, initial_balance: f64) -> Self {
        Self {
            config,
            fee_rate,
            initial_balance,
        }
    }

    pub fn run(&self, klines: &[Kline]) -> anyhow::Result<BacktestResult> {
        if klines.is_empty() {
            anyhow::bail!("No kline data to backtest");
        }

        let mut calculator = IndicatorCalculator::new(
            self.config.strategy.ema_short,
            self.config.strategy.ema_long,
            self.config.strategy.rsi_period,
            self.config.strategy.bollinger_period,
            self.config.strategy.bollinger_std,
        )?;

        let strategy = ScalpingStrategy::new(self.config.strategy.clone());
        let mut risk_manager = RiskManager::new(self.config.trading.clone(), self.initial_balance);

        let mut balance = self.initial_balance;
        let mut position: Option<Position> = None;
        let mut entry_reason = String::new();
        let mut trades: Vec<BacktestTrade> = Vec::new();
        let mut equity_curve: Vec<f64> = vec![balance];
        let mut last_reset_day: Option<u32> = None;

        let start_time = ms_to_datetime(klines.first().unwrap().open_time);
        let end_time = ms_to_datetime(klines.last().unwrap().close_time);

        for kline in klines {
            let price = kline.close;
            if price <= 0.0 {
                continue;
            }

            let candle_time = ms_to_datetime(kline.open_time);
            let today = candle_time.ordinal();

            // Daily reset
            match last_reset_day {
                Some(day) if day != today => {
                    risk_manager.reset_daily();
                    last_reset_day = Some(today);
                }
                None => {
                    last_reset_day = Some(today);
                }
                _ => {}
            }

            // Update indicators
            let indicators = calculator.update(price);

            // Generate signal
            let signal = strategy.evaluate(&indicators, position.as_ref());

            match signal {
                Signal::Buy(ref info) => {
                    if position.is_none() && risk_manager.can_trade() {
                        let quantity = self.config.trading.quantity;
                        if risk_manager.check_position_size(quantity) {
                            let cost = price * quantity;
                            let entry_fee = cost * self.fee_rate;
                            if balance >= cost + entry_fee {
                                balance -= cost + entry_fee;
                                position = Some(Position::new(price, quantity, candle_time));
                                entry_reason = info.reason.clone();
                            }
                        }
                    }
                }
                Signal::Sell(ref info) => {
                    if let Some(pos) = position.take() {
                        let exit_value = price * pos.quantity;
                        let exit_fee = exit_value * self.fee_rate;
                        let entry_cost = pos.entry_price * pos.quantity;
                        let entry_fee = entry_cost * self.fee_rate;
                        let total_fee = entry_fee + exit_fee;
                        let pnl = exit_value - entry_cost - total_fee;
                        let pnl_pct = pnl / entry_cost * 100.0;

                        balance += exit_value - exit_fee;
                        risk_manager.record_trade(pnl);

                        trades.push(BacktestTrade {
                            entry_price: pos.entry_price,
                            exit_price: price,
                            quantity: pos.quantity,
                            entry_time: pos.entry_time,
                            exit_time: candle_time,
                            pnl,
                            pnl_pct,
                            fee: total_fee,
                            reason: format!("{} -> {}", entry_reason, info.reason),
                        });

                        equity_curve.push(balance);
                    }
                }
                Signal::Hold => {}
            }
        }

        // Force-close any open position at last candle's close price
        if let Some(pos) = position.take() {
            let last_price = klines.last().unwrap().close;
            let exit_value = last_price * pos.quantity;
            let exit_fee = exit_value * self.fee_rate;
            let entry_cost = pos.entry_price * pos.quantity;
            let entry_fee = entry_cost * self.fee_rate;
            let total_fee = entry_fee + exit_fee;
            let pnl = exit_value - entry_cost - total_fee;
            let pnl_pct = pnl / entry_cost * 100.0;

            balance += exit_value - exit_fee;

            trades.push(BacktestTrade {
                entry_price: pos.entry_price,
                exit_price: last_price,
                quantity: pos.quantity,
                entry_time: pos.entry_time,
                exit_time: end_time,
                pnl,
                pnl_pct,
                fee: total_fee,
                reason: format!("{} -> forced close (end of data)", entry_reason),
            });

            equity_curve.push(balance);
        }

        info!(
            "Backtest complete: {} trades, final balance: {:.2}",
            trades.len(),
            balance
        );

        Ok(BacktestResult::calculate(
            self.config.strategy.symbol.clone(),
            self.config.strategy.interval.clone(),
            start_time,
            end_time,
            klines.len(),
            self.initial_balance,
            balance,
            trades,
            equity_curve,
        ))
    }
}

fn ms_to_datetime(ms: u64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(ms as i64).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    fn test_config() -> AppConfig {
        AppConfig {
            exchange: ExchangeConfig {
                base_url: "https://api.binance.com".to_string(),
                ws_url: "wss://stream.binance.com:9443".to_string(),
                testnet: false,
            },
            strategy: StrategyConfig {
                symbol: "BTCUSDT".to_string(),
                interval: "1m".to_string(),
                ema_short: 9,
                ema_long: 21,
                rsi_period: 14,
                rsi_oversold: 30.0,
                rsi_overbought: 70.0,
                bollinger_period: 20,
                bollinger_std: 2.0,
                stop_loss_pct: 0.3,
                take_profit_pct: 0.5,
            },
            trading: TradingConfig {
                quantity: 0.1,
                max_position: 1.0,
                stop_loss_pct: 0.3,
                take_profit_pct: 0.5,
                max_daily_trades: 100,
                max_daily_loss_pct: 5.0,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                trade_log_path: "trades.csv".to_string(),
            },
            dashboard: DashboardConfig::default(),
            telegram: TelegramConfig::default(),
        }
    }

    fn make_klines_stable(count: usize, base_price: f64) -> Vec<Kline> {
        let start_ms = 1704067200000u64; // 2024-01-01 00:00:00 UTC
        (0..count)
            .map(|i| {
                let open_time = start_ms + (i as u64) * 60_000;
                Kline {
                    open_time,
                    open: base_price,
                    high: base_price + 1.0,
                    low: base_price - 1.0,
                    close: base_price,
                    volume: 100.0,
                    close_time: open_time + 59_999,
                }
            })
            .collect()
    }

    fn make_klines_with_prices(prices: &[f64]) -> Vec<Kline> {
        let start_ms = 1704067200000u64;
        prices
            .iter()
            .enumerate()
            .map(|(i, &price)| {
                let open_time = start_ms + (i as u64) * 60_000;
                Kline {
                    open_time,
                    open: price,
                    high: price + 0.5,
                    low: price - 0.5,
                    close: price,
                    volume: 100.0,
                    close_time: open_time + 59_999,
                }
            })
            .collect()
    }

    #[test]
    fn test_empty_klines() {
        let engine = BacktestEngine::new(test_config(), 0.001, 10000.0);
        let result = engine.run(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_stable_price_no_trades() {
        // With a perfectly stable price, no EMA crossover should occur → no trades
        let klines = make_klines_stable(200, 50000.0);
        let engine = BacktestEngine::new(test_config(), 0.001, 10000.0);
        let result = engine.run(&klines).unwrap();
        assert_eq!(result.trades.len(), 0);
        assert!((result.final_balance - 10000.0).abs() < 1e-9);
    }

    #[test]
    fn test_fee_deduction() {
        // Generate prices that cause a crossover then exit
        let mut prices = vec![50000.0; 50];
        // Drop to create short EMA below long
        for i in 0..30 {
            prices.push(49900.0 - i as f64 * 2.0);
        }
        // Rise sharply → short EMA crosses above long → BUY signal
        for i in 0..30 {
            prices.push(49900.0 + i as f64 * 10.0);
        }
        // Then keep rising to trigger take profit or other sell
        for i in 0..50 {
            prices.push(50200.0 + i as f64 * 5.0);
        }

        let klines = make_klines_with_prices(&prices);
        let engine = BacktestEngine::new(test_config(), 0.001, 100000.0);
        let result = engine.run(&klines).unwrap();

        // If there are trades, fees should be > 0
        for trade in &result.trades {
            assert!(trade.fee > 0.0, "Expected fee > 0 for each trade");
        }
        if !result.trades.is_empty() {
            assert!(result.total_fees > 0.0);
        }
    }

    #[test]
    fn test_balance_conservation() {
        // With zero fees, total PnL should equal balance change
        let mut prices = vec![50000.0; 50];
        for i in 0..30 {
            prices.push(49900.0 - i as f64 * 2.0);
        }
        for i in 0..30 {
            prices.push(49900.0 + i as f64 * 10.0);
        }
        for i in 0..50 {
            prices.push(50200.0 + i as f64 * 5.0);
        }

        let klines = make_klines_with_prices(&prices);
        let engine = BacktestEngine::new(test_config(), 0.0, 100000.0);
        let result = engine.run(&klines).unwrap();

        let total_pnl: f64 = result.trades.iter().map(|t| t.pnl).sum();
        let balance_change = result.final_balance - result.initial_balance;
        assert!(
            (total_pnl - balance_change).abs() < 0.01,
            "PnL sum ({:.4}) should match balance change ({:.4})",
            total_pnl,
            balance_change
        );
    }

    #[test]
    fn test_equity_curve_starts_with_initial() {
        let klines = make_klines_stable(100, 50000.0);
        let engine = BacktestEngine::new(test_config(), 0.001, 10000.0);
        let result = engine.run(&klines).unwrap();
        assert_eq!(result.equity_curve[0], 10000.0);
    }

    #[test]
    fn test_ms_to_datetime() {
        let dt = ms_to_datetime(1704067200000);
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }
}
