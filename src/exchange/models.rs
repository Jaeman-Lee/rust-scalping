use serde::{Deserialize, Serialize};

// --- WebSocket Kline ---

#[derive(Debug, Clone, Deserialize)]
pub struct WsKlineEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: WsKline,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsKline {
    #[serde(rename = "t")]
    pub open_time: u64,
    #[serde(rename = "T")]
    pub close_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub interval: String,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "c")]
    pub close: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "x")]
    pub is_closed: bool,
}

impl WsKline {
    pub fn close_f64(&self) -> f64 {
        self.close.parse().unwrap_or(0.0)
    }

    pub fn high_f64(&self) -> f64 {
        self.high.parse().unwrap_or(0.0)
    }

    pub fn low_f64(&self) -> f64 {
        self.low.parse().unwrap_or(0.0)
    }

    pub fn volume_f64(&self) -> f64 {
        self.volume.parse().unwrap_or(0.0)
    }
}

// --- REST API ---

#[derive(Debug, Clone, Deserialize)]
pub struct ServerTime {
    #[serde(rename = "serverTime")]
    pub server_time: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfo {
    pub balances: Vec<Balance>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

impl Balance {
    pub fn free_f64(&self) -> f64 {
        self.free.parse().unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NewOrder {
    pub symbol: String,
    pub side: OrderSide,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub quantity: Option<String>,
    pub price: Option<String>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<String>,
    #[serde(rename = "newOrderRespType")]
    pub new_order_resp_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderSide {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "BUY"),
            OrderSide::Sell => write!(f, "SELL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    #[serde(rename = "MARKET")]
    Market,
    #[serde(rename = "LIMIT")]
    Limit,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Market => write!(f, "MARKET"),
            OrderType::Limit => write!(f, "LIMIT"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderResponse {
    pub symbol: String,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: Option<String>,
    pub price: Option<String>,
    #[serde(rename = "origQty")]
    pub orig_qty: Option<String>,
    #[serde(rename = "executedQty")]
    pub executed_qty: Option<String>,
    pub status: Option<String>,
    pub side: Option<String>,
    #[serde(rename = "type")]
    pub order_type: Option<String>,
}

// --- Kline (REST) ---

#[derive(Debug, Clone)]
pub struct Kline {
    pub open_time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub close_time: u64,
}
