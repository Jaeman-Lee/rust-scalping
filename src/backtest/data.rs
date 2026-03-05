use crate::exchange::client::BinanceClient;
use crate::exchange::models::Kline;
use tracing::info;

pub async fn fetch_klines_paginated(
    client: &BinanceClient,
    symbol: &str,
    interval: &str,
    start_ms: u64,
    end_ms: u64,
) -> anyhow::Result<Vec<Kline>> {
    let mut all_klines: Vec<Kline> = Vec::new();
    let mut current_start = start_ms;
    let limit = 1000u32;

    info!(
        "Fetching historical klines for {} ({}) from {} to {}",
        symbol, interval, start_ms, end_ms
    );

    loop {
        if current_start >= end_ms {
            break;
        }

        let batch = client
            .get_klines_range(symbol, interval, current_start, end_ms, limit)
            .await?;

        if batch.is_empty() {
            break;
        }

        let batch_len = batch.len();
        let last_close_time = batch.last().unwrap().close_time;
        all_klines.extend(batch);

        info!(
            "Fetched {} candles (total: {})",
            batch_len,
            all_klines.len()
        );

        if batch_len < limit as usize {
            break;
        }

        current_start = last_close_time + 1;

        // Rate limit: 200ms between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    info!(
        "Finished fetching {} total candles for {}",
        all_klines.len(),
        symbol
    );

    Ok(all_klines)
}
