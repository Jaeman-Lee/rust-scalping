use crate::config::AppConfig;
use crate::dashboard::SharedState;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
    #[command(description = "Show bot status, price, position, indicators")]
    Status,
    #[command(description = "Show balance, daily PnL, trade count")]
    Balance,
    #[command(description = "Show last 5 trades")]
    Trades,
    #[command(description = "Show daily PnL summary and win rate")]
    Pnl,
    #[command(description = "Resume trading (unpause)")]
    StartBot,
    #[command(description = "Pause trading")]
    StopBot,
    #[command(description = "Show current config")]
    Config,
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    shared_state: SharedState,
    config: AppConfig,
) -> ResponseResult<()> {
    let text = match cmd {
        Command::Status => format_status(&shared_state).await,
        Command::Balance => format_balance(&shared_state).await,
        Command::Trades => format_trades(&shared_state).await,
        Command::Pnl => format_pnl(&shared_state).await,
        Command::StartBot => {
            let mut state = shared_state.write().await;
            state.is_paused = false;
            "Trading resumed.".to_string()
        }
        Command::StopBot => {
            let mut state = shared_state.write().await;
            state.is_paused = true;
            "Trading paused.".to_string()
        }
        Command::Config => format_config(&config),
    };

    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}

async fn format_status(state: &SharedState) -> String {
    let s = state.read().await;
    let mut text = format!(
        "Status\n\
         Symbol: {}\n\
         Price: {:.2}\n\
         Running: {} | Paused: {}\n",
        s.symbol,
        s.current_price,
        if s.is_running { "Yes" } else { "No" },
        if s.is_paused { "Yes" } else { "No" },
    );

    if let Some(ref ind) = s.indicators {
        text.push_str(&format!(
            "\nIndicators:\n\
             EMA(9): {:.2} | EMA(21): {:.2}\n\
             RSI: {:.1}\n\
             BB: {:.2} / {:.2} / {:.2}\n",
            ind.ema_short, ind.ema_long, ind.rsi, ind.bb_upper, ind.bb_middle, ind.bb_lower,
        ));
    }

    if let Some(ref pos) = s.position {
        text.push_str(&format!(
            "\nPosition:\n\
             Entry: {:.2} | Qty: {:.6}\n\
             PnL: {:.4} ({:.2}%)\n",
            pos.entry_price, pos.quantity, pos.unrealized_pnl, pos.unrealized_pnl_pct,
        ));
    } else {
        text.push_str("\nNo open position\n");
    }

    text
}

async fn format_balance(state: &SharedState) -> String {
    let s = state.read().await;
    let risk = &s.risk;
    format!(
        "Balance\n\
         Account: {:.2} USDT\n\
         Daily PnL: {:.4}\n\
         Daily Trades: {}/{}\n\
         Win Rate: {:.1}% ({}/{})\n",
        risk.account_balance,
        risk.daily_pnl,
        risk.daily_trades,
        risk.max_daily_trades,
        risk.win_rate(),
        risk.total_wins,
        risk.total_wins + risk.total_losses,
    )
}

async fn format_trades(state: &SharedState) -> String {
    let s = state.read().await;
    if s.recent_trades.is_empty() {
        return "No recent trades.".to_string();
    }

    let mut text = "Recent Trades:\n".to_string();
    for trade in s.recent_trades.iter().rev().take(5) {
        text.push_str(&format!(
            "{} | {} {:.6} @ {:.2}",
            trade.side, trade.quantity, trade.entry_price, trade.exit_price,
        ));
        if trade.side == "SELL" {
            text.push_str(&format!(" | PnL: {:.4} ({:.2}%)", trade.pnl, trade.pnl_pct));
        }
        text.push('\n');
    }
    text
}

async fn format_pnl(state: &SharedState) -> String {
    let s = state.read().await;
    let risk = &s.risk;
    format!(
        "Daily PnL Summary\n\
         PnL: {:.4}\n\
         Trades: {}\n\
         Wins: {} | Losses: {}\n\
         Win Rate: {:.1}%\n\
         Consecutive Losses: {}\n",
        risk.daily_pnl,
        risk.daily_trades,
        risk.total_wins,
        risk.total_losses,
        risk.win_rate(),
        risk.consecutive_losses,
    )
}

fn format_config(config: &AppConfig) -> String {
    format!(
        "Config\n\
         Symbol: {}\n\
         Interval: {}\n\
         EMA: {}/{}\n\
         RSI Period: {}\n\
         BB: {}/{:.1}\n\
         Quantity: {:.6}\n\
         SL: {:.1}% | TP: {:.1}%\n\
         Max Daily Trades: {}\n\
         Max Daily Loss: {:.1}%\n",
        config.strategy.symbol,
        config.strategy.interval,
        config.strategy.ema_short,
        config.strategy.ema_long,
        config.strategy.rsi_period,
        config.strategy.bollinger_period,
        config.strategy.bollinger_std,
        config.trading.quantity,
        config.trading.stop_loss_pct,
        config.trading.take_profit_pct,
        config.trading.max_daily_trades,
        config.trading.max_daily_loss_pct,
    )
}
