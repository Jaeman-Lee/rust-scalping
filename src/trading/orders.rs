use crate::exchange::client::BinanceClient;
use crate::exchange::models::{OrderResponse, OrderSide};
use tracing::info;

pub struct OrderManager<'a> {
    client: &'a BinanceClient,
    symbol: String,
}

#[allow(dead_code)]
impl<'a> OrderManager<'a> {
    pub fn new(client: &'a BinanceClient, symbol: &str) -> Self {
        Self {
            client,
            symbol: symbol.to_string(),
        }
    }

    pub async fn market_buy(&self, quantity: f64) -> anyhow::Result<OrderResponse> {
        info!("Executing market BUY: {:.6} {}", quantity, self.symbol);
        self.client
            .place_market_order(&self.symbol, OrderSide::Buy, quantity)
            .await
    }

    pub async fn market_sell(&self, quantity: f64) -> anyhow::Result<OrderResponse> {
        info!("Executing market SELL: {:.6} {}", quantity, self.symbol);
        self.client
            .place_market_order(&self.symbol, OrderSide::Sell, quantity)
            .await
    }

    pub async fn limit_buy(&self, quantity: f64, price: f64) -> anyhow::Result<OrderResponse> {
        info!(
            "Executing limit BUY: {:.6} {} @ {:.2}",
            quantity, self.symbol, price
        );
        self.client
            .place_limit_order(&self.symbol, OrderSide::Buy, quantity, price)
            .await
    }

    pub async fn limit_sell(&self, quantity: f64, price: f64) -> anyhow::Result<OrderResponse> {
        info!(
            "Executing limit SELL: {:.6} {} @ {:.2}",
            quantity, self.symbol, price
        );
        self.client
            .place_limit_order(&self.symbol, OrderSide::Sell, quantity, price)
            .await
    }

    pub async fn cancel(&self, order_id: u64) -> anyhow::Result<()> {
        self.client.cancel_order(&self.symbol, order_id).await?;
        Ok(())
    }
}
