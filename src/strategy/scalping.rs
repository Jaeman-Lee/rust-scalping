use crate::config::StrategyConfig;
use crate::indicators::calculator::IndicatorValues;
use crate::strategy::signals::{Signal, SignalInfo};
use crate::trading::position::Position;
use chrono::Utc;

pub struct ScalpingStrategy {
    config: StrategyConfig,
}

impl ScalpingStrategy {
    pub fn new(config: StrategyConfig) -> Self {
        Self { config }
    }

    /// Evaluate indicators and current position to generate a trading signal.
    pub fn evaluate(&self, indicators: &IndicatorValues, position: Option<&Position>) -> Signal {
        // If we have a position, check exit conditions first
        if let Some(pos) = position {
            if let Some(sell_signal) = self.check_exit(indicators, pos) {
                return sell_signal;
            }
        }

        // If no position, check entry conditions
        if position.is_none() {
            if let Some(buy_signal) = self.check_entry(indicators) {
                return buy_signal;
            }
        }

        Signal::Hold
    }

    /// Check buy conditions (all must be met):
    /// 1. EMA(short) crosses above EMA(long)
    /// 2. RSI < overbought threshold
    /// 3. Price near lower Bollinger Band or crossing above middle band
    fn check_entry(&self, ind: &IndicatorValues) -> Option<Signal> {
        // Need previous values for crossover detection
        let (prev_short, prev_long) = match (ind.prev_ema_short, ind.prev_ema_long) {
            (Some(ps), Some(pl)) => (ps, pl),
            _ => return None,
        };

        // Condition 1: EMA crossover (short crosses above long)
        let ema_cross_up = prev_short <= prev_long && ind.ema_short > ind.ema_long;

        // Condition 2: RSI not overbought
        let rsi_ok = ind.rsi < self.config.rsi_overbought;

        // Condition 3: Price near lower BB or crossing above middle BB
        let bb_range = ind.bb_upper - ind.bb_lower;
        let near_lower_bb = (ind.price - ind.bb_lower) < bb_range * 0.3;
        let above_middle_bb = ind.price > ind.bb_middle;
        let bb_ok = near_lower_bb || above_middle_bb;

        if ema_cross_up && rsi_ok && bb_ok {
            let reason = format!(
                "EMA cross up ({}>{:.0}), RSI={:.1}, BB_pos={}",
                self.config.ema_short,
                ind.ema_long,
                ind.rsi,
                if near_lower_bb {
                    "near_lower"
                } else {
                    "above_mid"
                }
            );
            return Some(Signal::Buy(SignalInfo {
                reason,
                price: ind.price,
                timestamp: Utc::now(),
            }));
        }

        None
    }

    /// Check sell conditions (any one triggers):
    /// 1. EMA(short) crosses below EMA(long)
    /// 2. RSI > overbought
    /// 3. Price reaches upper Bollinger Band
    /// 4. Stop loss (-0.3%)
    /// 5. Take profit (+0.5%)
    fn check_exit(&self, ind: &IndicatorValues, position: &Position) -> Option<Signal> {
        let entry_price = position.entry_price;
        let pnl_pct = (ind.price - entry_price) / entry_price * 100.0;

        // Condition 4: Stop loss
        if pnl_pct <= -self.config.stop_loss_pct {
            return Some(Signal::Sell(SignalInfo {
                reason: format!("Stop loss hit: {:.2}%", pnl_pct),
                price: ind.price,
                timestamp: Utc::now(),
            }));
        }

        // Condition 5: Take profit
        if pnl_pct >= self.config.take_profit_pct {
            return Some(Signal::Sell(SignalInfo {
                reason: format!("Take profit hit: {:.2}%", pnl_pct),
                price: ind.price,
                timestamp: Utc::now(),
            }));
        }

        // Condition 1: EMA cross down
        if let (Some(prev_short), Some(prev_long)) = (ind.prev_ema_short, ind.prev_ema_long) {
            if prev_short >= prev_long && ind.ema_short < ind.ema_long {
                return Some(Signal::Sell(SignalInfo {
                    reason: format!("EMA cross down, PnL: {:.2}%", pnl_pct),
                    price: ind.price,
                    timestamp: Utc::now(),
                }));
            }
        }

        // Condition 2: RSI overbought
        if ind.rsi > self.config.rsi_overbought {
            return Some(Signal::Sell(SignalInfo {
                reason: format!("RSI overbought: {:.1}, PnL: {:.2}%", ind.rsi, pnl_pct),
                price: ind.price,
                timestamp: Utc::now(),
            }));
        }

        // Condition 3: Price at upper Bollinger Band
        let bb_range = ind.bb_upper - ind.bb_lower;
        if bb_range > 0.0 && (ind.bb_upper - ind.price) < bb_range * 0.05 {
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
            stop_loss_pct: 0.3,
            take_profit_pct: 0.5,
        }
    }

    #[test]
    fn test_hold_without_crossover() {
        let strategy = ScalpingStrategy::new(default_config());
        let ind = IndicatorValues {
            ema_short: 100.0,
            ema_long: 101.0,
            prev_ema_short: Some(99.0),
            prev_ema_long: Some(101.0),
            rsi: 50.0,
            bb_upper: 105.0,
            bb_middle: 100.0,
            bb_lower: 95.0,
            price: 100.0,
        };
        let signal = strategy.evaluate(&ind, None);
        assert_eq!(signal, Signal::Hold);
    }

    #[test]
    fn test_stop_loss() {
        let strategy = ScalpingStrategy::new(default_config());
        let position = Position::new(100.0, 0.001, chrono::Utc::now());
        let ind = IndicatorValues {
            ema_short: 99.0,
            ema_long: 99.5,
            prev_ema_short: Some(99.0),
            prev_ema_long: Some(99.5),
            rsi: 50.0,
            bb_upper: 105.0,
            bb_middle: 100.0,
            bb_lower: 95.0,
            price: 99.5, // -0.5% from entry
        };
        let signal = strategy.evaluate(&ind, Some(&position));
        match signal {
            Signal::Sell(info) => assert!(info.reason.contains("Stop loss")),
            _ => panic!("Expected sell signal for stop loss"),
        }
    }

    #[test]
    fn test_take_profit() {
        let strategy = ScalpingStrategy::new(default_config());
        let position = Position::new(100.0, 0.001, Utc::now());
        let ind = IndicatorValues {
            ema_short: 101.0,
            ema_long: 100.5,
            prev_ema_short: Some(101.0),
            prev_ema_long: Some(100.5),
            rsi: 50.0,
            bb_upper: 105.0,
            bb_middle: 100.0,
            bb_lower: 95.0,
            price: 100.6, // +0.6% from entry → exceeds take_profit_pct=0.5
        };
        let signal = strategy.evaluate(&ind, Some(&position));
        match signal {
            Signal::Sell(info) => assert!(info.reason.contains("Take profit")),
            _ => panic!("Expected sell signal for take profit"),
        }
    }

    #[test]
    fn test_buy_signal_ema_crossover() {
        let strategy = ScalpingStrategy::new(default_config());
        // EMA short was below long, now crosses above (bullish crossover)
        let ind = IndicatorValues {
            ema_short: 101.0, // now above long
            ema_long: 100.5,
            prev_ema_short: Some(99.0), // was below long
            prev_ema_long: Some(100.0),
            rsi: 50.0, // not overbought
            bb_upper: 105.0,
            bb_middle: 100.0,
            bb_lower: 95.0,
            price: 101.0, // above middle BB
        };
        let signal = strategy.evaluate(&ind, None);
        match signal {
            Signal::Buy(info) => assert!(info.reason.contains("EMA cross up")),
            _ => panic!("Expected buy signal, got {:?}", signal),
        }
    }

    #[test]
    fn test_no_buy_when_rsi_overbought() {
        let strategy = ScalpingStrategy::new(default_config());
        // EMA crossover happens but RSI is overbought
        let ind = IndicatorValues {
            ema_short: 101.0,
            ema_long: 100.5,
            prev_ema_short: Some(99.0),
            prev_ema_long: Some(100.0),
            rsi: 75.0, // overbought (> 70)
            bb_upper: 105.0,
            bb_middle: 100.0,
            bb_lower: 95.0,
            price: 101.0,
        };
        let signal = strategy.evaluate(&ind, None);
        assert_eq!(signal, Signal::Hold);
    }

    #[test]
    fn test_sell_on_rsi_overbought() {
        let strategy = ScalpingStrategy::new(default_config());
        let position = Position::new(100.0, 0.001, Utc::now());
        let ind = IndicatorValues {
            ema_short: 100.2,
            ema_long: 100.1,
            prev_ema_short: Some(100.2),
            prev_ema_long: Some(100.1),
            rsi: 75.0, // overbought
            bb_upper: 105.0,
            bb_middle: 100.0,
            bb_lower: 95.0,
            price: 100.2, // slightly profitable but not take profit
        };
        let signal = strategy.evaluate(&ind, Some(&position));
        match signal {
            Signal::Sell(info) => assert!(info.reason.contains("RSI overbought")),
            _ => panic!("Expected sell on RSI overbought"),
        }
    }

    #[test]
    fn test_sell_on_ema_cross_down() {
        let strategy = ScalpingStrategy::new(default_config());
        let position = Position::new(100.0, 0.001, Utc::now());
        // EMA short crosses below long (bearish)
        let ind = IndicatorValues {
            ema_short: 99.8, // now below long
            ema_long: 100.0,
            prev_ema_short: Some(100.1), // was above long
            prev_ema_long: Some(100.0),
            rsi: 50.0,
            bb_upper: 105.0,
            bb_middle: 100.0,
            bb_lower: 95.0,
            price: 100.1, // small profit, within SL/TP range
        };
        let signal = strategy.evaluate(&ind, Some(&position));
        match signal {
            Signal::Sell(info) => assert!(info.reason.contains("EMA cross down")),
            _ => panic!("Expected sell on EMA cross down"),
        }
    }

    #[test]
    fn test_sell_on_bb_upper() {
        let strategy = ScalpingStrategy::new(default_config());
        let position = Position::new(100.0, 0.001, Utc::now());
        let ind = IndicatorValues {
            ema_short: 100.2,
            ema_long: 100.1,
            prev_ema_short: Some(100.2),
            prev_ema_long: Some(100.1),
            rsi: 65.0,
            bb_upper: 100.4,
            bb_middle: 100.0,
            bb_lower: 99.6,
            price: 100.39, // very close to BB upper (within 5% of range)
        };
        let signal = strategy.evaluate(&ind, Some(&position));
        match signal {
            Signal::Sell(info) => assert!(info.reason.contains("BB upper")),
            _ => panic!("Expected sell on BB upper, got {:?}", signal),
        }
    }

    #[test]
    fn test_hold_with_position_when_no_exit_conditions() {
        let strategy = ScalpingStrategy::new(default_config());
        let position = Position::new(100.0, 0.001, Utc::now());
        let ind = IndicatorValues {
            ema_short: 100.1,
            ema_long: 100.0,
            prev_ema_short: Some(100.1),
            prev_ema_long: Some(100.0),
            rsi: 55.0,
            bb_upper: 105.0,
            bb_middle: 100.0,
            bb_lower: 95.0,
            price: 100.2, // small profit, no exit trigger
        };
        let signal = strategy.evaluate(&ind, Some(&position));
        assert_eq!(signal, Signal::Hold);
    }

    #[test]
    fn test_no_buy_without_prev_values() {
        let strategy = ScalpingStrategy::new(default_config());
        let ind = IndicatorValues {
            ema_short: 101.0,
            ema_long: 100.0,
            prev_ema_short: None, // no previous data
            prev_ema_long: None,
            rsi: 50.0,
            bb_upper: 105.0,
            bb_middle: 100.0,
            bb_lower: 95.0,
            price: 101.0,
        };
        let signal = strategy.evaluate(&ind, None);
        assert_eq!(signal, Signal::Hold);
    }
}
