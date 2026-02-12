use chrono::{DateTime, Utc};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Position {
    pub entry_price: f64,
    pub quantity: f64,
    pub entry_time: DateTime<Utc>,
}

impl Position {
    pub fn new(entry_price: f64, quantity: f64, entry_time: DateTime<Utc>) -> Self {
        Self {
            entry_price,
            quantity,
            entry_time,
        }
    }

    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        (current_price - self.entry_price) * self.quantity
    }

    pub fn unrealized_pnl_pct(&self, current_price: f64) -> f64 {
        (current_price - self.entry_price) / self.entry_price * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_position(entry: f64, qty: f64) -> Position {
        Position::new(entry, qty, Utc::now())
    }

    #[test]
    fn test_profit_pnl() {
        let pos = make_position(100.0, 0.5);
        // Price goes up 10%
        let pnl = pos.unrealized_pnl(110.0);
        assert!((pnl - 5.0).abs() < 1e-9); // 10 * 0.5
    }

    #[test]
    fn test_loss_pnl() {
        let pos = make_position(100.0, 1.0);
        let pnl = pos.unrealized_pnl(95.0);
        assert!((pnl - (-5.0)).abs() < 1e-9);
    }

    #[test]
    fn test_pnl_pct_profit() {
        let pos = make_position(100.0, 1.0);
        let pct = pos.unrealized_pnl_pct(100.5);
        assert!((pct - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_pnl_pct_loss() {
        let pos = make_position(100.0, 1.0);
        let pct = pos.unrealized_pnl_pct(99.7);
        assert!((pct - (-0.3)).abs() < 1e-9);
    }

    #[test]
    fn test_zero_pnl() {
        let pos = make_position(100.0, 1.0);
        assert!((pos.unrealized_pnl(100.0)).abs() < 1e-9);
        assert!((pos.unrealized_pnl_pct(100.0)).abs() < 1e-9);
    }
}
