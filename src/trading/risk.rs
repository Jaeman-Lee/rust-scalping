use crate::config::TradingConfig;
use tracing::{info, warn};

pub struct RiskManager {
    config: TradingConfig,
    daily_trades: u32,
    daily_pnl: f64,
    consecutive_losses: u32,
    account_balance: f64,
}

impl RiskManager {
    pub fn new(config: TradingConfig, account_balance: f64) -> Self {
        Self {
            config,
            daily_trades: 0,
            daily_pnl: 0.0,
            consecutive_losses: 0,
            account_balance,
        }
    }

    /// Check if a new trade is allowed
    pub fn can_trade(&self) -> bool {
        if self.daily_trades >= self.config.max_daily_trades {
            warn!(
                "Daily trade limit reached: {}/{}",
                self.daily_trades, self.config.max_daily_trades
            );
            return false;
        }

        let max_loss = self.account_balance * self.config.max_daily_loss_pct / 100.0;
        if self.daily_pnl < -max_loss {
            warn!(
                "Daily loss limit reached: {:.2} (max: -{:.2})",
                self.daily_pnl, max_loss
            );
            return false;
        }

        if self.consecutive_losses >= 5 {
            warn!(
                "Too many consecutive losses: {}. Trading paused.",
                self.consecutive_losses
            );
            return false;
        }

        true
    }

    /// Check if position size is within limits
    pub fn check_position_size(&self, quantity: f64) -> bool {
        if quantity > self.config.max_position {
            warn!(
                "Position size {:.6} exceeds max {:.6}",
                quantity, self.config.max_position
            );
            return false;
        }
        true
    }

    /// Record a completed trade
    pub fn record_trade(&mut self, pnl: f64) {
        self.daily_trades += 1;
        self.daily_pnl += pnl;

        if pnl < 0.0 {
            self.consecutive_losses += 1;
        } else {
            self.consecutive_losses = 0;
        }

        info!(
            "Trade recorded: PnL={:.4}, Daily PnL={:.4}, Trades={}/{}",
            pnl, self.daily_pnl, self.daily_trades, self.config.max_daily_trades
        );
    }

    /// Reset daily counters (call at start of new trading day)
    pub fn reset_daily(&mut self) {
        info!(
            "Resetting daily counters. Final stats: trades={}, pnl={:.4}",
            self.daily_trades, self.daily_pnl
        );
        self.daily_trades = 0;
        self.daily_pnl = 0.0;
        self.consecutive_losses = 0;
    }

    pub fn update_balance(&mut self, balance: f64) {
        self.account_balance = balance;
    }

    pub fn daily_pnl(&self) -> f64 {
        self.daily_pnl
    }

    pub fn daily_trades(&self) -> u32 {
        self.daily_trades
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_trading_config() -> TradingConfig {
        TradingConfig {
            quantity: 0.001,
            max_position: 0.01,
            stop_loss_pct: 0.3,
            take_profit_pct: 0.5,
            max_daily_trades: 5,
            max_daily_loss_pct: 2.0,
        }
    }

    #[test]
    fn test_can_trade_initially() {
        let rm = RiskManager::new(default_trading_config(), 1000.0);
        assert!(rm.can_trade());
    }

    #[test]
    fn test_daily_trade_limit() {
        let mut rm = RiskManager::new(default_trading_config(), 1000.0);
        for _ in 0..5 {
            rm.record_trade(1.0); // 5 winning trades
        }
        assert!(!rm.can_trade()); // 6th should be blocked
    }

    #[test]
    fn test_daily_loss_limit() {
        let mut rm = RiskManager::new(default_trading_config(), 1000.0);
        // max_daily_loss_pct=2.0, balance=1000 → max loss = 20.0
        rm.record_trade(-25.0); // exceeds -20
        assert!(!rm.can_trade());
    }

    #[test]
    fn test_consecutive_losses_limit() {
        let mut rm = RiskManager::new(default_trading_config(), 100000.0);
        // 5 consecutive losses (small amounts to not hit daily loss limit)
        for _ in 0..5 {
            rm.record_trade(-0.01);
        }
        assert!(!rm.can_trade());
    }

    #[test]
    fn test_consecutive_losses_reset_on_win() {
        let config = TradingConfig {
            max_daily_trades: 100, // high limit so trade count doesn't interfere
            ..default_trading_config()
        };
        let mut rm = RiskManager::new(config, 100000.0);
        for _ in 0..4 {
            rm.record_trade(-0.01);
        }
        rm.record_trade(0.01); // win resets consecutive loss counter
        assert!(rm.can_trade());
    }

    #[test]
    fn test_position_size_check() {
        let rm = RiskManager::new(default_trading_config(), 1000.0);
        assert!(rm.check_position_size(0.005));
        assert!(rm.check_position_size(0.01)); // exact max
        assert!(!rm.check_position_size(0.02)); // over max
    }

    #[test]
    fn test_reset_daily() {
        let mut rm = RiskManager::new(default_trading_config(), 1000.0);
        rm.record_trade(-5.0);
        rm.record_trade(-5.0);
        assert_eq!(rm.daily_trades(), 2);
        assert!((rm.daily_pnl() - (-10.0)).abs() < 1e-9);

        rm.reset_daily();
        assert_eq!(rm.daily_trades(), 0);
        assert!((rm.daily_pnl()).abs() < 1e-9);
        assert!(rm.can_trade());
    }

    #[test]
    fn test_record_trade_updates_pnl() {
        let mut rm = RiskManager::new(default_trading_config(), 1000.0);
        rm.record_trade(10.0);
        rm.record_trade(-3.0);
        assert!((rm.daily_pnl() - 7.0).abs() < 1e-9);
        assert_eq!(rm.daily_trades(), 2);
    }
}
