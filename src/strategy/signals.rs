use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Buy(SignalInfo),
    Sell(SignalInfo),
    Hold,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SignalInfo {
    pub reason: String,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Signal::Buy(info) => {
                write!(f, "BUY @ {:.2} ({})", info.price, info.reason)
            }
            Signal::Sell(info) => {
                write!(f, "SELL @ {:.2} ({})", info.price, info.reason)
            }
            Signal::Hold => write!(f, "HOLD"),
        }
    }
}
