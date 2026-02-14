use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::VecDeque;

/// Engine state shared between trading engine, dashboard, and telegram
#[derive(Debug, Clone, Serialize)]
pub struct EngineState {
    pub current_price: f64,
    pub symbol: String,
    pub indicators: Option<IndicatorSnapshot>,
    pub position: Option<PositionSnapshot>,
    pub risk: RiskSnapshot,
    pub recent_trades: VecDeque<TradeSnapshot>,
    pub is_running: bool,
    pub is_paused: bool,
    pub last_update: DateTime<Utc>,
}

impl EngineState {
    pub fn new(symbol: String) -> Self {
        Self {
            current_price: 0.0,
            symbol,
            indicators: None,
            position: None,
            risk: RiskSnapshot::default(),
            recent_trades: VecDeque::with_capacity(101),
            is_running: true,
            is_paused: false,
            last_update: Utc::now(),
        }
    }

    pub fn push_trade(&mut self, trade: TradeSnapshot) {
        if self.recent_trades.len() >= 100 {
            self.recent_trades.pop_front();
        }
        self.recent_trades.push_back(trade);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IndicatorSnapshot {
    pub ema_short: f64,
    pub ema_long: f64,
    pub rsi: f64,
    pub bb_upper: f64,
    pub bb_middle: f64,
    pub bb_lower: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PositionSnapshot {
    pub entry_price: f64,
    pub quantity: f64,
    pub entry_time: DateTime<Utc>,
    pub unrealized_pnl: f64,
    pub unrealized_pnl_pct: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct RiskSnapshot {
    pub daily_trades: u32,
    pub daily_pnl: f64,
    pub consecutive_losses: u32,
    pub account_balance: f64,
    pub max_daily_trades: u32,
    pub max_daily_loss_pct: f64,
    pub total_wins: u32,
    pub total_losses: u32,
}

impl RiskSnapshot {
    pub fn win_rate(&self) -> f64 {
        let total = self.total_wins + self.total_losses;
        if total == 0 {
            0.0
        } else {
            self.total_wins as f64 / total as f64 * 100.0
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TradeSnapshot {
    pub side: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub timestamp: DateTime<Utc>,
}

/// Events broadcast to dashboard and telegram
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum DashboardEvent {
    PriceUpdate {
        price: f64,
        symbol: String,
        indicators: Option<IndicatorSnapshot>,
    },
    TradeExecuted {
        trade: TradeSnapshot,
    },
    RiskAlert {
        message: String,
    },
    EngineStatusChanged {
        is_running: bool,
        is_paused: bool,
    },
}
