pub mod handlers;
pub mod server;
pub mod state;

use state::DashboardEvent;
use state::EngineState;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub type SharedState = Arc<RwLock<EngineState>>;
pub type EventSender = broadcast::Sender<DashboardEvent>;
