use crate::exchange::auth;
use crate::exchange::models::*;
use reqwest::Client;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info};

pub struct BinanceClient {
    client: Client,
    base_url: String,
    api_key: String,
    secret_key: String,
}

#[allow(dead_code)]
impl BinanceClient {
    pub fn new(base_url: &str, api_key: String, secret_key: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            api_key,
            secret_key,
        }
    }

    fn timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64
    }

    pub async fn server_time(&self) -> anyhow::Result<u64> {
        let url = format!("{}/api/v3/time", self.base_url);
        let resp: ServerTime = self.client.get(&url).send().await?.json().await?;
        Ok(resp.server_time)
    }

    pub async fn account_info(&self) -> anyhow::Result<AccountInfo> {
        let timestamp = Self::timestamp_ms().to_string();
        let params = [("timestamp", timestamp.as_str()), ("recvWindow", "5000")];
        let query = auth::build_signed_query(&self.secret_key, &params);
        let url = format!("{}/api/v3/account?{}", self.base_url, query);

        let resp = self
            .client
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("Account info failed: {} - {}", status, body);
            anyhow::bail!("Account info request failed: {} - {}", status, body);
        }

        let info: AccountInfo = resp.json().await?;
        Ok(info)
    }

    pub async fn place_market_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
    ) -> anyhow::Result<OrderResponse> {
        let timestamp = Self::timestamp_ms().to_string();
        let qty_str = format!("{:.6}", quantity);
        let side_str = side.to_string();

        let params = [
            ("symbol", symbol),
            ("side", &side_str),
            ("type", "MARKET"),
            ("quantity", &qty_str),
            ("newOrderRespType", "FULL"),
            ("recvWindow", "5000"),
            ("timestamp", &timestamp),
        ];

        let query = auth::build_signed_query(&self.secret_key, &params);
        let url = format!("{}/api/v3/order?{}", self.base_url, query);

        info!(
            "Placing market {} order: {} {} @ market",
            side_str, qty_str, symbol
        );

        let resp = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("Order failed: {} - {}", status, body);
            anyhow::bail!("Order request failed: {} - {}", status, body);
        }

        let order: OrderResponse = resp.json().await?;
        info!("Order placed successfully: order_id={}", order.order_id);
        Ok(order)
    }

    pub async fn place_limit_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: f64,
    ) -> anyhow::Result<OrderResponse> {
        let timestamp = Self::timestamp_ms().to_string();
        let qty_str = format!("{:.6}", quantity);
        let price_str = format!("{:.2}", price);
        let side_str = side.to_string();

        let params = [
            ("symbol", symbol),
            ("side", &side_str),
            ("type", "LIMIT"),
            ("timeInForce", "GTC"),
            ("quantity", &qty_str),
            ("price", &price_str),
            ("newOrderRespType", "FULL"),
            ("recvWindow", "5000"),
            ("timestamp", &timestamp),
        ];

        let query = auth::build_signed_query(&self.secret_key, &params);
        let url = format!("{}/api/v3/order?{}", self.base_url, query);

        info!(
            "Placing limit {} order: {} {} @ {}",
            side_str, qty_str, symbol, price_str
        );

        let resp = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("Order failed: {} - {}", status, body);
            anyhow::bail!("Order request failed: {} - {}", status, body);
        }

        let order: OrderResponse = resp.json().await?;
        info!("Order placed successfully: order_id={}", order.order_id);
        Ok(order)
    }

    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: u64,
    ) -> anyhow::Result<serde_json::Value> {
        let timestamp = Self::timestamp_ms().to_string();
        let order_id_str = order_id.to_string();

        let params = [
            ("symbol", symbol),
            ("orderId", order_id_str.as_str()),
            ("recvWindow", "5000"),
            ("timestamp", timestamp.as_str()),
        ];

        let query = auth::build_signed_query(&self.secret_key, &params);
        let url = format!("{}/api/v3/order?{}", self.base_url, query);

        debug!("Cancelling order {} for {}", order_id, symbol);

        let resp = self
            .client
            .delete(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("Cancel order failed: {} - {}", status, body);
            anyhow::bail!("Cancel order request failed: {} - {}", status, body);
        }

        let result: serde_json::Value = resp.json().await?;
        info!("Order {} cancelled successfully", order_id);
        Ok(result)
    }

    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<Kline>> {
        let url = format!(
            "{}/api/v3/klines?symbol={}&interval={}&limit={}",
            self.base_url, symbol, interval, limit
        );

        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Get klines failed: {} - {}", status, body);
        }

        let raw: Vec<Vec<serde_json::Value>> = resp.json().await?;
        let klines: Vec<Kline> = raw
            .into_iter()
            .filter_map(|k| {
                if k.len() < 7 {
                    return None;
                }
                Some(Kline {
                    open_time: k[0].as_u64()?,
                    open: k[1].as_str()?.parse().ok()?,
                    high: k[2].as_str()?.parse().ok()?,
                    low: k[3].as_str()?.parse().ok()?,
                    close: k[4].as_str()?.parse().ok()?,
                    volume: k[5].as_str()?.parse().ok()?,
                    close_time: k[6].as_u64()?,
                })
            })
            .collect();

        Ok(klines)
    }
}
