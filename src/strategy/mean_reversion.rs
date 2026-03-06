use crate::config::StrategyConfig;
use crate::indicators::calculator::IndicatorValues;
use crate::strategy::signals::{Signal, SignalInfo};
use crate::trading::position::Position;
use chrono::Utc;

/// Bollinger Bands Mean Reversion Strategy
///
/// Buy: RSI < oversold (30) AND price <= lower BB
/// Sell (any one):
///   1. Stop loss: PnL <= -stop_loss_pct
///   2. Take profit: price >= middle BB (mean reversion target)
///   3. RSI > overbought (70)
///   4. Price >= upper BB
pub struct MeanReversionStrategy {
    config: StrategyConfig,
}

impl MeanReversionStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        Self { config }
    }

    pub fn evaluate(&self, indicators: &IndicatorValues, position: Option<&Position>) -> Signal {
        if let Some(pos) = position {
            if let Some(sell_signal) = self.check_exit(indicators, pos) {
                return sell_signal;
            }
        }

        if position.is_none() {
            if let Some(buy_signal) = self.check_entry(indicators) {
                return buy_signal;
            }
        }

        Signal::Hold
    }

    fn check_entry(&self, ind: &IndicatorValues) -> Option<Signal> {
        // Need at least some indicator history
        ind.prev_ema_short?;

        // Condition 1: RSI is oversold
        let rsi_oversold = ind.rsi < self.config.rsi_oversold;

        // Condition 2: Price at or below lower Bollinger Band
        let at_lower_bb = ind.price <= ind.bb_lower;

        if rsi_oversold && at_lower_bb {
            let reason = format!(
                "Mean reversion: RSI={:.1} (oversold), price={:.2} <= BB_lower={:.2}",
                ind.rsi, ind.price, ind.bb_lower
            );
            return Some(Signal::Buy(SignalInfo {
                reason,
                price: ind.price,
                timestamp: Utc::now(),
            }));
        }

        None
    }

    fn check_exit(&self, ind: &IndicatorValues, position: &Position) -> Option<Signal> {
        let entry_price = position.entry_price;
        let pnl_pct = (ind.price - entry_price) / entry_price * 100.0;

        // 1. Stop loss
        if pnl_pct <= -self.config.stop_loss_pct {
            return Some(Signal::Sell(SignalInfo {
                reason: format!("Stop loss hit: {:.2}%", pnl_pct),
                price: ind.price,
                timestamp: Utc::now(),
            }));
        }

        // 2. Mean reversion target: price reaches middle BB
        if ind.price >= ind.bb_middle {
            return Some(Signal::Sell(SignalInfo {
                reason: format!(
                    "Mean reversion target (middle BB={:.2}), PnL: {:.2}%",
                    ind.bb_middle, pnl_pct
                ),
                price: ind.price,
                timestamp: Utc::now(),
            }));
        }

        // 3. RSI overbought
        if ind.rsi > self.config.rsi_overbought {
            return Some(Signal::Sell(SignalInfo {
                reason: format!("RSI overbought: {:.1}, PnL: {:.2}%", ind.rsi, pnl_pct),
                price: ind.price,
                timestamp: Utc::now(),
            }));
        }

        // 4. Price at upper BB
        if ind.price >= ind.bb_upper {
            return Some(Signal::Sell(SignalInfo {
                reason: format!("BB upper reached, PnL: {:.2}%", pnl_pct),
                price: ind.price,
                timestamp: Utc::now(),
            }));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> StrategyConfig {
        StrategyConfig {
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            ema_short: 9,
            ema_long: 21,
            rsi_period: 14,
            rsi_oversold: 30.0,
            rsi_overbought: 70.0,
            bollinger_period: 20,
            bollinger_std: 2.0,
            stop_loss_pct: 0.5,
            take_profit_pct: 0.5,
            strategy_type: "mean_reversion".to_string(),
        }
    }

    fn make_indicators(
        price: f64,
        rsi: f64,
        bb_lower: f64,
        bb_middle: f64,
        bb_upper: f64,
    ) -> IndicatorValues {
        IndicatorValues {
            ema_short: price,
            ema_long: price,
            prev_ema_short: Some(price),
            prev_ema_long: Some(price),
            rsi,
            bb_upper,
            bb_middle,
            bb_lower,
            price,
        }
    }

    #[test]
    fn test_buy_oversold_at_lower_bb() {
        let strategy = MeanReversionStrategy::new(default_config());
        let ind = make_indicators(95.0, 25.0, 95.5, 100.0, 104.5);
        match strategy.evaluate(&ind, None) {
            Signal::Buy(info) => assert!(info.reason.contains("Mean reversion")),
            other => panic!("Expected Buy, got {:?}", other),
        }
    }

    #[test]
    fn test_no_buy_rsi_not_oversold() {
        let strategy = MeanReversionStrategy::new(default_config());
        let ind = make_indicators(95.0, 45.0, 95.5, 100.0, 104.5);
        assert_eq!(strategy.evaluate(&ind, None), Signal::Hold);
    }

    #[test]
    fn test_no_buy_price_above_lower_bb() {
        let strategy = MeanReversionStrategy::new(default_config());
        let ind = make_indicators(97.0, 25.0, 95.5, 100.0, 104.5);
        assert_eq!(strategy.evaluate(&ind, None), Signal::Hold);
    }

    #[test]
    fn test_sell_at_middle_bb() {
        let strategy = MeanReversionStrategy::new(default_config());
        let position = Position::new(95.0, 0.1, Utc::now());
        let ind = make_indicators(100.0, 50.0, 95.0, 100.0, 105.0);
        match strategy.evaluate(&ind, Some(&position)) {
            Signal::Sell(info) => assert!(info.reason.contains("Mean reversion target")),
            other => panic!("Expected Sell at middle BB, got {:?}", other),
        }
    }

    #[test]
    fn test_sell_stop_loss() {
        let strategy = MeanReversionStrategy::new(default_config());
        let position = Position::new(100.0, 0.1, Utc::now());
        let ind = make_indicators(99.0, 25.0, 98.0, 100.0, 102.0);
        match strategy.evaluate(&ind, Some(&position)) {
            Signal::Sell(info) => assert!(info.reason.contains("Stop loss")),
            other => panic!("Expected stop loss sell, got {:?}", other),
        }
    }

    #[test]
    fn test_hold_between_bands() {
        let strategy = MeanReversionStrategy::new(default_config());
        let position = Position::new(95.0, 0.1, Utc::now());
        // Price below middle BB, RSI normal, no stop loss
        let ind = make_indicators(97.0, 45.0, 94.0, 100.0, 106.0);
        assert_eq!(strategy.evaluate(&ind, Some(&position)), Signal::Hold);
    }

    #[test]
    fn test_no_buy_without_prev_values() {
        let strategy = MeanReversionStrategy::new(default_config());
        let ind = IndicatorValues {
            ema_short: 95.0,
            ema_long: 95.0,
            prev_ema_short: None,
            prev_ema_long: None,
            rsi: 25.0,
            bb_upper: 105.0,
            bb_middle: 100.0,
            bb_lower: 96.0,
            price: 95.0,
        };
        assert_eq!(strategy.evaluate(&ind, None), Signal::Hold);
    }
}
