use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub exchange: ExchangeConfig,
    pub strategy: StrategyConfig,
    pub trading: TradingConfig,
    pub logging: LoggingConfig,
    #[serde(default)]
    pub dashboard: DashboardConfig,
    #[serde(default)]
    pub telegram: TelegramConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeConfig {
    pub base_url: String,
    pub ws_url: String,
    pub testnet: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct StrategyConfig {
    pub symbol: String,
    pub interval: String,
    pub ema_short: usize,
    pub ema_long: usize,
    pub rsi_period: usize,
    pub rsi_oversold: f64,
    pub rsi_overbought: f64,
    pub bollinger_period: usize,
    pub bollinger_std: f64,
    #[serde(default = "default_stop_loss")]
    pub stop_loss_pct: f64,
    #[serde(default = "default_take_profit")]
    pub take_profit_pct: f64,
}

fn default_stop_loss() -> f64 {
    0.3
}

fn default_take_profit() -> f64 {
    0.5
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct TradingConfig {
    pub quantity: f64,
    pub max_position: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub max_daily_trades: u32,
    pub max_daily_loss_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub trade_log_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DashboardConfig {
    pub enabled: bool,
    pub port: u16,
    pub host: String,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 3001,
            host: "0.0.0.0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct TelegramConfig {
    pub enabled: bool,
}

impl AppConfig {
    pub fn load(config_path: &str) -> anyhow::Result<Self> {
        let path = Path::new(config_path);
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file '{}': {}", config_path, e))?;
        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file '{}': {}", config_path, e))?;
        Ok(config)
    }

    pub fn api_key() -> anyhow::Result<String> {
        std::env::var("BINANCE_API_KEY")
            .map_err(|_| anyhow::anyhow!("BINANCE_API_KEY environment variable not set"))
    }

    pub fn secret_key() -> anyhow::Result<String> {
        std::env::var("BINANCE_SECRET_KEY")
            .map_err(|_| anyhow::anyhow!("BINANCE_SECRET_KEY environment variable not set"))
    }

    pub fn telegram_bot_token() -> anyhow::Result<String> {
        std::env::var("TELEGRAM_BOT_TOKEN")
            .map_err(|_| anyhow::anyhow!("TELEGRAM_BOT_TOKEN environment variable not set"))
    }

    pub fn telegram_chat_id() -> anyhow::Result<i64> {
        std::env::var("TELEGRAM_CHAT_ID")
            .map_err(|_| anyhow::anyhow!("TELEGRAM_CHAT_ID environment variable not set"))?
            .parse::<i64>()
            .map_err(|e| anyhow::anyhow!("TELEGRAM_CHAT_ID is not a valid integer: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_config() {
        let config = AppConfig::load("config/default.toml");
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.strategy.symbol, "BTCUSDT");
        assert_eq!(config.strategy.ema_short, 9);
        assert_eq!(config.strategy.ema_long, 21);
        assert!(!config.exchange.testnet);
    }

    #[test]
    fn test_load_testnet_config() {
        let config = AppConfig::load("config/testnet.toml");
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.exchange.testnet);
        assert!(config.exchange.base_url.contains("testnet"));
    }

    #[test]
    fn test_load_nonexistent_config() {
        let config = AppConfig::load("config/nonexistent.toml");
        assert!(config.is_err());
    }

    #[test]
    fn test_trading_config_values() {
        let config = AppConfig::load("config/default.toml").unwrap();
        assert!((config.trading.quantity - 0.001).abs() < 1e-9);
        assert!((config.trading.stop_loss_pct - 0.3).abs() < 1e-9);
        assert!((config.trading.take_profit_pct - 0.5).abs() < 1e-9);
        assert_eq!(config.trading.max_daily_trades, 100);
    }
}
