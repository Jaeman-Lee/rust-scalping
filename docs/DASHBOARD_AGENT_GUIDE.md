# Rust Scalping Bot - Claude Agent 가이드

> 최종 업데이트: 2026-03-06
> 이 문서는 AI agent가 프로젝트를 즉시 이해하고 작업을 이어갈 수 있도록 작성됨.

---

## 프로젝트 진행 상태

### 완료 (검증됨)

| 항목 | 파일 | 검증 |
|------|------|------|
| 매매 엔진 + SharedState 통합 | `src/trading/engine.rs` | cargo test 67/67 |
| Axum REST API (5 엔드포인트) | `src/dashboard/handlers.rs`, `server.rs` | curl 테스트 OK |
| WebSocket 이벤트 스트림 | `src/dashboard/handlers.rs::ws_handler` | 구조 완료 |
| Next.js 프론트엔드 | `dashboard/` | npm run build OK, dev 서버 OK |
| 텔레그램 봇 코드 | `src/telegram/` | 컴파일 OK |
| 설정 구조 (dashboard/telegram) | `src/config.rs` | serde(default) 적용 |
| 백테스트 모듈 | `src/backtest/` | cargo test 19개 (엔진 6 + 메트릭 13) |
| Mean Reversion 전략 | `src/strategy/mean_reversion.rs` | cargo test 7개 |
| 테스트넷 E2E | - | 2026-02-14 실행 검증 완료 |

### 다음 작업: 텔레그램 봇 실제 연동

**현재 상태**: `src/telegram/` 코드는 완성됨. 사용자가 BotFather에서 토큰을 발급받고 `.env`에 설정하면 동작함.

**연동 시 확인할 사항**:
1. `.env`에 `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID` 설정
2. `config/testnet.toml`에 `[telegram] enabled = true` 설정
3. 봇 재시작 후 `/status` 명령어로 응답 확인
4. 매매 체결 시 자동 알림 수신 확인
5. `/stop_bot` → `/start_bot` 제어 동작 확인

**연동 후 추가 개선 가능 사항**:
- Chat ID 인증 (허용된 사용자만 명령 실행)
- 알림 레벨 설정 (매매만 / 전체)
- 마크다운 포맷 메시지 (teloxide `ParseMode::MarkdownV2`)
- 그룹 채팅 지원

---

## 아키텍처

```
[Live Mode]
TradingEngine ──writes──> Arc<RwLock<EngineState>> <──reads── Axum REST API
      │                                                       Axum WebSocket
      │                                                       Telegram commands
      └──sends──> broadcast<DashboardEvent> ──subscribes──> WS clients
                                                             Telegram alerts

[Backtest Mode]
BinanceClient.get_klines_range() → fetch_klines_paginated() → Vec<Kline>
  → BacktestEngine.run(klines)
    → IndicatorCalculator + Strategy (Scalping/MeanReversion) + RiskManager (재사용)
    → 시뮬레이션 루프 (수수료 차감, 일일 리셋)
    → BacktestResult → Display (터미널 출력) + CSV (선택)
```

### 데이터 흐름

```
WebSocket(kline) → broadcast → TradingEngine
  → IndicatorCalculator.update(price)
  → is_paused 체크 (텔레그램 /stop_bot 제어)
  → ScalpingStrategy.evaluate(indicators, position)
  → Signal::Buy/Sell/Hold
  → RiskManager.can_trade()
  → OrderManager.market_buy/sell()
  → TradeLogger.log_trade()
  → update_shared_state()
    → SharedState write (가격, 지표, 포지션, 리스크)
    → DashboardEvent::PriceUpdate broadcast
    → DashboardEvent::TradeExecuted broadcast (매매 시)
```

---

## 모듈 구조

```
src/
├── main.rs              # 엔트리포인트 + CLI 서브커맨드 (Trade/Backtest) + spawn
├── config.rs            # AppConfig + DashboardConfig + TelegramConfig
├── backtest/
│   ├── mod.rs           # 모듈 선언
│   ├── data.rs          # 페이지네이션 kline 수집 (1000개씩, 200ms 딜레이)
│   ├── engine.rs        # BacktestEngine (시뮬레이션 루프 + 수수료 + 일일 리셋)
│   └── metrics.rs       # BacktestResult, BacktestTrade, 지표 계산, Display/CSV
├── dashboard/
│   ├── mod.rs           # SharedState = Arc<RwLock<EngineState>>, EventSender 타입
│   ├── state.rs         # EngineState, DashboardEvent, *Snapshot 타입들
│   ├── server.rs        # Axum 서버 (CORS, graceful shutdown)
│   └── handlers.rs      # REST 핸들러 + WebSocket upgrade
├── telegram/
│   ├── mod.rs           # 모듈 선언
│   ├── bot.rs           # teloxide 디스패처 + alert listener spawn
│   ├── commands.rs      # Command enum + handle_command() + format_*()
│   └── alerts.rs        # DashboardEvent 구독 → bot.send_message()
├── exchange/
│   ├── auth.rs          # HMAC-SHA256 서명
│   ├── client.rs        # BinanceClient (REST) + get_klines_range()
│   ├── models.rs        # API 타입들
│   └── websocket.rs     # kline WebSocket 스트림
├── indicators/
│   └── calculator.rs    # EMA, RSI, BollingerBands 래퍼
├── strategy/
│   ├── scalping.rs      # Scalping 전략 (EMA 크로스오버)
│   ├── mean_reversion.rs # Mean Reversion 전략 (BB + RSI 과매도)
│   └── signals.rs       # Signal enum
├── trading/
│   ├── engine.rs        # TradingEngine (SharedState/EventSender 통합)
│   ├── orders.rs        # OrderManager
│   ├── position.rs      # Position (PnL 계산)
│   └── risk.rs          # RiskManager (한도 관리 + public getters)
└── utils/
    └── logger.rs        # CSV 로깅 + tracing 초기화

dashboard/               # Next.js 14 + TypeScript + Tailwind
├── app/
│   ├── layout.tsx       # 네비게이션 (Dashboard/Trades/Settings)
│   ├── page.tsx         # 메인: 차트 + 4개 카드 + 거래 테이블
│   ├── trades/page.tsx  # 거래 내역 (limit=100)
│   └── settings/page.tsx # 설정 조회 (읽기 전용)
├── components/
│   ├── PriceChart.tsx   # Canvas 기반 실시간 차트
│   ├── PositionCard.tsx # 포지션 정보
│   ├── DailyStats.tsx   # 일일 통계
│   ├── IndicatorPanel.tsx # 지표 패널
│   ├── RiskStatus.tsx   # 리스크 프로그레스바
│   ├── TradeTable.tsx   # 거래 내역 테이블
│   └── ConnectionStatus.tsx # WS 연결 상태
├── hooks/
│   ├── useWebSocket.ts  # 자동 재연결 + DashboardEvent 파싱
│   └── useApi.ts        # REST 폴링 래퍼
├── next.config.js       # API 프록시 (/api → localhost:3001)
└── package.json         # next 14, tailwindcss 3, lightweight-charts 4
```

---

## 핵심 타입 정의

### EngineState (`src/dashboard/state.rs`)
```rust
pub struct EngineState {
    pub current_price: f64,
    pub symbol: String,
    pub indicators: Option<IndicatorSnapshot>,
    pub position: Option<PositionSnapshot>,
    pub risk: RiskSnapshot,
    pub recent_trades: VecDeque<TradeSnapshot>,  // 최대 100건
    pub is_running: bool,
    pub is_paused: bool,                         // 텔레그램 /stop_bot 제어
    pub last_update: DateTime<Utc>,
}
```

### DashboardEvent (`src/dashboard/state.rs`)
```rust
#[serde(tag = "type")]
pub enum DashboardEvent {
    PriceUpdate { price: f64, symbol: String, indicators: Option<IndicatorSnapshot> },
    TradeExecuted { trade: TradeSnapshot },
    RiskAlert { message: String },
    EngineStatusChanged { is_running: bool, is_paused: bool },
}
```

### TradingEngine 시그니처 (`src/trading/engine.rs`)
```rust
pub fn new(
    config: AppConfig,
    client: BinanceClient,
    trade_logger: TradeLogger,
    shared_state: SharedState,     // Arc<RwLock<EngineState>>
    event_tx: EventSender,         // broadcast::Sender<DashboardEvent>
) -> anyhow::Result<Self>
```

### CLI 구조 (`src/main.rs`)
```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,                          // None → Trade 모드 (하위 호환)
    #[arg(short, long, default_value = "config/default.toml", global = true)]
    config: String,
    #[arg(long, default_value_t = false, global = true)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Command {
    Trade,
    Backtest {
        start: String,              // YYYY-MM-DD
        end: String,                // YYYY-MM-DD
        output: Option<String>,     // CSV 파일 경로
        fee_rate: f64,              // 수수료율 (%, 기본 0.1)
        initial_balance: f64,       // 초기 잔고 (기본 10000.0)
    },
}
```

### BacktestEngine (`src/backtest/engine.rs`)
```rust
pub struct BacktestEngine {
    config: AppConfig,
    fee_rate: f64,         // 0.001 = 0.1%
    initial_balance: f64,
}

impl BacktestEngine {
    pub fn run(&self, klines: &[Kline]) -> anyhow::Result<BacktestResult>
}
```

**시뮬레이션 루프**: 기존 `TradingEngine::process_kline` 로직 미러링
- `IndicatorCalculator` + `StrategyKind` (Scalping/MeanReversion) + `RiskManager` 재사용
- 전략 선택: `config.strategy.strategy_type` → `StrategyKind` enum dispatch
- 수수료: 매수/매도 양쪽 적용 (`entry_fee + exit_fee = price * qty * fee_rate * 2`)
- 일일 리셋: 캔들 날짜 변경 시 `risk_manager.reset_daily()`
- 종료: 미청산 포지션 강제 청산

### BacktestResult (`src/backtest/metrics.rs`)
```rust
pub struct BacktestResult {
    pub trades: Vec<BacktestTrade>,
    pub equity_curve: Vec<f64>,
    pub total_return_pct: f64,
    pub win_rate: f64,
    pub profit_factor: f64,       // 총 이익 / |총 손실|
    pub max_drawdown_pct: f64,    // equity curve 기반
    pub sharpe_ratio: f64,        // 연간화
    pub total_fees: f64,
    // ... 기타 메타데이터
}
```

---

## 설정 구조 (`src/config.rs`)

```rust
pub struct AppConfig {
    pub exchange: ExchangeConfig,
    pub strategy: StrategyConfig,
    pub trading: TradingConfig,
    pub logging: LoggingConfig,
    #[serde(default)]
    pub dashboard: DashboardConfig,  // enabled, port, host
    #[serde(default)]
    pub telegram: TelegramConfig,    // enabled
}
```

환경변수 로더: `AppConfig::telegram_bot_token()`, `AppConfig::telegram_chat_id()`

---

## 의존성

| 크레이트 | 버전 | 용도 |
|---------|------|------|
| `tokio` | 1 | 비동기 런타임 |
| `reqwest` | 0.12 | Binance REST API |
| `tokio-tungstenite` | 0.24 | Binance WebSocket |
| `ta` | 0.5.0 | EMA, RSI, BollingerBands |
| `hmac` + `sha2` | 0.12/0.10 | HMAC-SHA256 서명 |
| `axum` | 0.7 (ws) | REST API + WebSocket 서버 |
| `tower-http` | 0.6 (cors, trace) | CORS, 트레이싱 |
| `teloxide` | 0.13 (macros) | 텔레그램 봇 |
| `clap` | 4 | CLI 파싱 |
| `tracing` | 0.1 | 구조화된 로깅 |
| `chrono` | 0.4 | 시간 처리 |
| `serde` + `serde_json` | 1 | 직렬화 |

---

## 빌드 & 테스트

```bash
export PATH="$HOME/.cargo/bin:$PATH"  # WSL 환경 필수

cargo test          # 67개 통과 (매매 41 + 백테스트 19 + mean_reversion 7)
cargo clippy        # 경고 0개
cargo build --release

# 프론트엔드
cd dashboard && npm install && npm run build
```

---

## 주의사항

- `SharedState` write lock: 엔진에서만 잡음 (예외: 텔레그램 `/start_bot`, `/stop_bot`)
- `recent_trades`: VecDeque, 100건 초과 시 pop_front
- broadcast 에러 무시: subscriber 0이면 send 실패하지만 정상
- WSL 환경에서 `cargo`는 `~/.cargo/bin/`에 있어 PATH 추가 필요
- Node.js: nvm으로 v20 설치됨 (`~/.nvm/`)
