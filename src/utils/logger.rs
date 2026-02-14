use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use tracing::info;

pub struct TradeLogger {
    path: PathBuf,
}

pub struct TradeRecord<'a> {
    pub symbol: &'a str,
    pub side: &'a str,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
}

impl TradeLogger {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let path = PathBuf::from(path);

        // Create CSV header if file doesn't exist
        if !path.exists() {
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&path)?;
            writeln!(
                file,
                "timestamp,symbol,side,entry_price,exit_price,quantity,pnl,pnl_pct"
            )?;
            info!("Created trade log file: {}", path.display());
        }

        Ok(Self { path })
    }

    pub fn log_trade(&self, record: &TradeRecord<'_>) -> anyhow::Result<()> {
        let mut file = OpenOptions::new().append(true).open(&self.path)?;
        writeln!(
            file,
            "{},{},{},{:.2},{:.2},{:.6},{:.4},{:.2}",
            Utc::now().to_rfc3339(),
            record.symbol,
            record.side,
            record.entry_price,
            record.exit_price,
            record.quantity,
            record.pnl,
            record.pnl_pct,
        )?;
        Ok(())
    }
}

pub fn init_tracing(level: &str) {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .init();
}
