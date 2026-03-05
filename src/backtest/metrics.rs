use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug, Clone)]
pub struct BacktestTrade {
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub fee: f64,
    pub reason: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BacktestResult {
    pub symbol: String,
    pub interval: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub candle_count: usize,
    pub initial_balance: f64,
    pub final_balance: f64,
    pub trades: Vec<BacktestTrade>,
    pub equity_curve: Vec<f64>,
    pub total_return_pct: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub max_drawdown_pct: f64,
    pub sharpe_ratio: f64,
    pub total_fees: f64,
}

impl BacktestResult {
    #[allow(clippy::too_many_arguments)]
    pub fn calculate(
        symbol: String,
        interval: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        candle_count: usize,
        initial_balance: f64,
        final_balance: f64,
        trades: Vec<BacktestTrade>,
        equity_curve: Vec<f64>,
    ) -> Self {
        let total_return_pct = if initial_balance > 0.0 {
            (final_balance - initial_balance) / initial_balance * 100.0
        } else {
            0.0
        };

        let total_trades = trades.len();
        let wins = trades.iter().filter(|t| t.pnl > 0.0).count();
        let win_rate = if total_trades > 0 {
            wins as f64 / total_trades as f64 * 100.0
        } else {
            0.0
        };

        let gross_profit: f64 = trades.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).sum();
        let gross_loss: f64 = trades
            .iter()
            .filter(|t| t.pnl < 0.0)
            .map(|t| t.pnl.abs())
            .sum();
        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        let max_drawdown_pct = calculate_max_drawdown(&equity_curve);
        let sharpe_ratio = calculate_sharpe_ratio(&trades);
        let total_fees: f64 = trades.iter().map(|t| t.fee).sum();

        Self {
            symbol,
            interval,
            start_time,
            end_time,
            candle_count,
            initial_balance,
            final_balance,
            trades,
            equity_curve,
            total_return_pct,
            win_rate,
            profit_factor,
            max_drawdown_pct,
            sharpe_ratio,
            total_fees,
        }
    }

    pub fn to_csv(&self) -> String {
        let mut csv = String::from(
            "entry_time,exit_time,entry_price,exit_price,quantity,pnl,pnl_pct,fee,reason\n",
        );
        for t in &self.trades {
            csv.push_str(&format!(
                "{},{},{:.2},{:.2},{:.6},{:.4},{:.4},{:.4},{}\n",
                t.entry_time.format("%Y-%m-%d %H:%M:%S"),
                t.exit_time.format("%Y-%m-%d %H:%M:%S"),
                t.entry_price,
                t.exit_price,
                t.quantity,
                t.pnl,
                t.pnl_pct,
                t.fee,
                t.reason,
            ));
        }
        csv
    }
}

fn calculate_max_drawdown(equity_curve: &[f64]) -> f64 {
    if equity_curve.is_empty() {
        return 0.0;
    }

    let mut peak = equity_curve[0];
    let mut max_dd = 0.0;

    for &equity in equity_curve {
        if equity > peak {
            peak = equity;
        }
        let dd = (peak - equity) / peak * 100.0;
        if dd > max_dd {
            max_dd = dd;
        }
    }

    max_dd
}

fn calculate_sharpe_ratio(trades: &[BacktestTrade]) -> f64 {
    if trades.len() < 2 {
        return 0.0;
    }

    let returns: Vec<f64> = trades.iter().map(|t| t.pnl_pct).collect();
    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();

    if std_dev < 1e-10 {
        return 0.0;
    }

    // Annualize assuming ~365*24*60 one-minute candles per year
    // Approximate annual trades from the sample
    let annualization_factor = (525_600.0 / n).sqrt();
    mean / std_dev * annualization_factor
}

impl fmt::Display for BacktestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_trades = self.trades.len();
        let wins = self.trades.iter().filter(|t| t.pnl > 0.0).count();
        let losses = total_trades - wins;
        let avg_win = if wins > 0 {
            self.trades
                .iter()
                .filter(|t| t.pnl > 0.0)
                .map(|t| t.pnl_pct)
                .sum::<f64>()
                / wins as f64
        } else {
            0.0
        };
        let avg_loss = if losses > 0 {
            self.trades
                .iter()
                .filter(|t| t.pnl <= 0.0)
                .map(|t| t.pnl_pct)
                .sum::<f64>()
                / losses as f64
        } else {
            0.0
        };

        writeln!(f)?;
        writeln!(f, "══════════════════════════════════════════")?;
        writeln!(f, "  BACKTEST RESULTS: {} ({})", self.symbol, self.interval)?;
        writeln!(
            f,
            "  Period: {} ~ {}",
            self.start_time.format("%Y-%m-%d"),
            self.end_time.format("%Y-%m-%d")
        )?;
        writeln!(f, "  Candles: {}", self.candle_count)?;
        writeln!(f, "══════════════════════════════════════════")?;
        writeln!(f, "  Initial Balance:  ${:.2}", self.initial_balance)?;
        writeln!(f, "  Final Balance:    ${:.2}", self.final_balance)?;
        writeln!(f, "  Total Return:     {:+.2}%", self.total_return_pct)?;
        writeln!(f, "──────────────────────────────────────────")?;
        writeln!(f, "  Total Trades:     {}", total_trades)?;
        writeln!(f, "  Wins / Losses:    {} / {}", wins, losses)?;
        writeln!(f, "  Win Rate:         {:.2}%", self.win_rate)?;
        writeln!(f, "  Avg Win:          {:+.4}%", avg_win)?;
        writeln!(f, "  Avg Loss:         {:+.4}%", avg_loss)?;
        writeln!(f, "  Profit Factor:    {:.2}", self.profit_factor)?;
        writeln!(f, "  Max Drawdown:     -{:.2}%", self.max_drawdown_pct)?;
        writeln!(f, "  Sharpe Ratio:     {:.2}", self.sharpe_ratio)?;
        writeln!(f, "  Total Fees:       ${:.2}", self.total_fees)?;
        writeln!(f, "══════════════════════════════════════════")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trade(pnl: f64, pnl_pct: f64, fee: f64) -> BacktestTrade {
        BacktestTrade {
            entry_price: 100.0,
            exit_price: if pnl >= 0.0 { 100.5 } else { 99.5 },
            quantity: 0.1,
            entry_time: Utc::now(),
            exit_time: Utc::now(),
            pnl,
            pnl_pct,
            fee,
            reason: "test".to_string(),
        }
    }

    #[test]
    fn test_empty_result() {
        let result = BacktestResult::calculate(
            "BTCUSDT".to_string(),
            "1m".to_string(),
            Utc::now(),
            Utc::now(),
            0,
            10000.0,
            10000.0,
            vec![],
            vec![10000.0],
        );
        assert_eq!(result.total_return_pct, 0.0);
        assert_eq!(result.win_rate, 0.0);
        assert_eq!(result.profit_factor, 0.0);
        assert_eq!(result.total_fees, 0.0);
    }

    #[test]
    fn test_win_rate() {
        let trades = vec![
            make_trade(10.0, 1.0, 0.2),
            make_trade(-5.0, -0.5, 0.2),
            make_trade(8.0, 0.8, 0.2),
        ];
        let result = BacktestResult::calculate(
            "BTCUSDT".to_string(),
            "1m".to_string(),
            Utc::now(),
            Utc::now(),
            100,
            10000.0,
            10013.0,
            trades,
            vec![10000.0, 10010.0, 10005.0, 10013.0],
        );
        assert!((result.win_rate - 66.66666666666667).abs() < 0.01);
    }

    #[test]
    fn test_profit_factor() {
        let trades = vec![
            make_trade(10.0, 1.0, 0.0),
            make_trade(-5.0, -0.5, 0.0),
            make_trade(15.0, 1.5, 0.0),
        ];
        let result = BacktestResult::calculate(
            "BTCUSDT".to_string(),
            "1m".to_string(),
            Utc::now(),
            Utc::now(),
            100,
            10000.0,
            10020.0,
            trades,
            vec![10000.0],
        );
        // profit_factor = 25.0 / 5.0 = 5.0
        assert!((result.profit_factor - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_profit_factor_no_losses() {
        let trades = vec![make_trade(10.0, 1.0, 0.0), make_trade(5.0, 0.5, 0.0)];
        let result = BacktestResult::calculate(
            "BTCUSDT".to_string(),
            "1m".to_string(),
            Utc::now(),
            Utc::now(),
            100,
            10000.0,
            10015.0,
            trades,
            vec![10000.0],
        );
        assert!(result.profit_factor.is_infinite());
    }

    #[test]
    fn test_max_drawdown() {
        let equity = vec![10000.0, 10500.0, 10200.0, 9800.0, 10100.0];
        let dd = calculate_max_drawdown(&equity);
        // Peak = 10500, trough = 9800 → dd = 700/10500 * 100 = 6.666...%
        assert!((dd - 6.666666666666667).abs() < 0.01);
    }

    #[test]
    fn test_max_drawdown_empty() {
        assert_eq!(calculate_max_drawdown(&[]), 0.0);
    }

    #[test]
    fn test_max_drawdown_monotonic_up() {
        let equity = vec![100.0, 110.0, 120.0, 130.0];
        assert_eq!(calculate_max_drawdown(&equity), 0.0);
    }

    #[test]
    fn test_sharpe_ratio_insufficient_data() {
        let trades = vec![make_trade(10.0, 1.0, 0.0)];
        assert_eq!(calculate_sharpe_ratio(&trades), 0.0);
    }

    #[test]
    fn test_sharpe_ratio_zero_std() {
        let trades = vec![
            make_trade(10.0, 1.0, 0.0),
            make_trade(10.0, 1.0, 0.0),
            make_trade(10.0, 1.0, 0.0),
        ];
        assert_eq!(calculate_sharpe_ratio(&trades), 0.0);
    }

    #[test]
    fn test_total_fees() {
        let trades = vec![
            make_trade(10.0, 1.0, 0.5),
            make_trade(-5.0, -0.5, 0.5),
            make_trade(8.0, 0.8, 0.5),
        ];
        let result = BacktestResult::calculate(
            "BTCUSDT".to_string(),
            "1m".to_string(),
            Utc::now(),
            Utc::now(),
            100,
            10000.0,
            10013.0,
            trades,
            vec![10000.0],
        );
        assert!((result.total_fees - 1.5).abs() < 1e-9);
    }

    #[test]
    fn test_csv_output() {
        let trades = vec![make_trade(10.0, 1.0, 0.2)];
        let result = BacktestResult::calculate(
            "BTCUSDT".to_string(),
            "1m".to_string(),
            Utc::now(),
            Utc::now(),
            100,
            10000.0,
            10010.0,
            trades,
            vec![10000.0],
        );
        let csv = result.to_csv();
        assert!(csv.starts_with("entry_time,exit_time,"));
        assert!(csv.contains("100.00"));
        assert!(csv.contains("test"));
    }

    #[test]
    fn test_total_return_pct() {
        let result = BacktestResult::calculate(
            "BTCUSDT".to_string(),
            "1m".to_string(),
            Utc::now(),
            Utc::now(),
            100,
            10000.0,
            10500.0,
            vec![],
            vec![10000.0, 10500.0],
        );
        assert!((result.total_return_pct - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_display_format() {
        let result = BacktestResult::calculate(
            "BTCUSDT".to_string(),
            "1m".to_string(),
            Utc::now(),
            Utc::now(),
            1000,
            10000.0,
            10100.0,
            vec![make_trade(100.0, 1.0, 0.5)],
            vec![10000.0, 10100.0],
        );
        let output = format!("{}", result);
        assert!(output.contains("BACKTEST RESULTS"));
        assert!(output.contains("BTCUSDT"));
        assert!(output.contains("Total Trades"));
    }
}
