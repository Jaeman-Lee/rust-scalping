# CLAUDE.md - Project Context for AI Agents

이 파일은 다른 Claude agent(또는 AI 도구)가 이 프로젝트를 즉시 이해하고 작업을 이어갈 수 있도록 작성되었습니다.

## 프로젝트 개요

- **이름**: rust-scalping (Binance 스캘핑 자동매매 봇)
- **언어**: Rust (edition 2021)
- **저장소**: git@github.com:Jaeman-Lee/rust-scalping.git
- **상태**: 백테스트 + 멀티전략 구현 완료, 테스트넷 실행 전 단계

## 빌드 & 테스트

```bash
# 빌드 (Rust 1.70+ 필요)
cargo build --release

# 테스트 (67개)
cargo test

# 린트
cargo clippy

# 포맷
cargo fmt --check

# 프론트엔드 (Node.js 필요)
cd dashboard && npm install && npm run dev
```

## 프로젝트 구조

```
src/
├── main.rs              # 엔트리포인트 (CLI 서브커맨드: Trade/Backtest + 그레이스풀 셧다운)
├── config.rs            # TOML 설정 로드 + 환경변수 (DashboardConfig, TelegramConfig 포함)
├── backtest/
│   ├── mod.rs           # 모듈 선언
│   ├── data.rs          # Binance API 페이지네이션 과거 데이터 수집
│   ├── engine.rs        # 시뮬레이션 엔진 (기존 컴포넌트 재사용, 수수료 반영)
│   └── metrics.rs       # 결과 집계 (승률, Profit Factor, MDD, Sharpe 등)
├── dashboard/
│   ├── mod.rs           # SharedState, EventSender 타입 alias
│   ├── state.rs         # EngineState, DashboardEvent, 스냅샷 타입들
│   ├── server.rs        # Axum 서버 설정 (CORS, graceful shutdown)
│   └── handlers.rs      # REST 핸들러 + WebSocket 업그레이드
├── telegram/
│   ├── mod.rs           # 모듈 선언
│   ├── bot.rs           # teloxide 디스패처 설정
│   ├── commands.rs      # /status, /balance 등 명령어 핸들러
│   └── alerts.rs        # DashboardEvent 구독 → 알림 전송
├── exchange/
│   ├── auth.rs          # HMAC-SHA256 서명 (Binance API 인증)
│   ├── client.rs        # REST API (주문/잔고/캔들 조회)
│   ├── models.rs        # API 요청/응답 타입 정의
│   └── websocket.rs     # WebSocket 실시간 kline 스트림
├── indicators/
│   └── calculator.rs    # 기술지표 래퍼 (EMA, RSI, 볼린저밴드)
├── strategy/
│   ├── scalping.rs      # 스캘핑 전략 (EMA 크로스오버 기반)
│   ├── mean_reversion.rs # 평균회귀 전략 (BB + RSI 기반)
│   └── signals.rs       # Signal enum (Buy/Sell/Hold)
├── trading/
│   ├── engine.rs        # 매매 엔진 (WebSocket→지표→시그널→주문 + SharedState 갱신)
│   ├── orders.rs        # 주문 관리 (시장가/지정가)
│   ├── position.rs      # 포지션 추적 (진입가, 수량, PnL)
│   └── risk.rs          # 리스크 관리 (일일한도, 연속손실, 포지션크기)
└── utils/
    └── logger.rs        # CSV 거래 기록 + tracing 초기화

dashboard/               # Next.js 프론트엔드 (별도 프로젝트)
├── app/                 # 페이지 (메인, 거래내역, 설정)
├── components/          # React 컴포넌트 (차트, 포지션, 지표 등)
├── hooks/               # useWebSocket, useApi
└── next.config.js       # API 프록시 (/api → localhost:3001)

docs/                    # 문서
├── DASHBOARD_USER_GUIDE.md   # 사용자 가이드
└── DASHBOARD_AGENT_GUIDE.md  # AI agent 가이드
```

## 아키텍처

```
TradingEngine ──writes──> Arc<RwLock<EngineState>> <──reads── Axum REST API
      │                                                       Axum WebSocket
      └──sends──> broadcast<DashboardEvent> ──subscribes──> WS clients
                                                             Telegram alerts
```

## 핵심 데이터 흐름

```
WebSocket(kline) → broadcast channel → TradingEngine
  → IndicatorCalculator.update(price)
  → ScalpingStrategy.evaluate(indicators, position)
  → Signal::Buy/Sell/Hold
  → RiskManager.can_trade() 체크
  → OrderManager.market_buy/sell()
  → TradeLogger.log_trade()
  → update_shared_state() → PriceUpdate/TradeExecuted 이벤트 발행
```

## 대시보드 API 엔드포인트

| 엔드포인트 | 설명 |
|-----------|------|
| `GET /api/status` | 전체 상태 (가격, 지표, 포지션, 리스크) |
| `GET /api/trades?limit=N` | 최근 거래 내역 |
| `GET /api/indicators` | 현재 기술지표 값 |
| `GET /api/balance` | 잔고, 일일 PnL, 승률 |
| `GET /api/ws` | WebSocket (DashboardEvent 실시간 스트림) |

## 텔레그램 명령어

| 명령어 | 기능 |
|--------|------|
| `/status` | 봇 상태, 가격, 포지션, 지표 |
| `/balance` | 잔고, 일일 PnL, 거래 수 |
| `/trades` | 최근 5건 거래 |
| `/pnl` | 일일 PnL 요약, 승률 |
| `/start_bot` | 매매 재개 |
| `/stop_bot` | 매매 일시정지 |
| `/config` | 현재 설정값 조회 |

## 매매 전략

설정 파일의 `strategy_type` 필드로 전략 선택 (기본값: `"scalping"`).

### 1. Scalping (EMA 크로스오버) — `strategy_type = "scalping"`

**매수 (3가지 모두 충족):**
1. EMA(9) > EMA(21) 크로스오버 (이전에는 아래였음)
2. RSI < 70
3. 가격이 볼린저밴드 하단 30% 이내 또는 중간선 위

**매도 (하나라도 충족):**
1. 손절: PnL ≤ -stop_loss_pct (기본 -0.3%)
2. 익절: PnL ≥ take_profit_pct (기본 +0.5%)
3. EMA(9) < EMA(21) 크로스다운
4. RSI > 70
5. 볼린저밴드 상단 5% 이내 도달

### 2. Mean Reversion (볼린저밴드 평균회귀) — `strategy_type = "mean_reversion"`

**매수 (2가지 모두 충족):**
1. RSI < rsi_oversold (기본 30, 과매도)
2. 가격 ≤ 볼린저밴드 하단

**매도 (하나라도 충족):**
1. 손절: PnL ≤ -stop_loss_pct
2. 가격 ≥ 볼린저밴드 중간선 (평균회귀 목표)
3. RSI > rsi_overbought (70)
4. 가격 ≥ 볼린저밴드 상단

## 설정 파일

- `config/default.toml` - 실거래 설정 (scalping 전략, api.binance.com)
- `config/testnet.toml` - 테스트넷 설정 (testnet.binance.vision)
- `config/mean_reversion.toml` - 평균회귀 전략 설정
- `.env` - API 키 + 텔레그램 토큰 (gitignore됨, `.env.example` 참고)

## 실행 방법

```bash
# .env 파일 필요
cp .env.example .env
# BINANCE_API_KEY, BINANCE_SECRET_KEY 입력
# (선택) TELEGRAM_BOT_TOKEN, TELEGRAM_CHAT_ID 입력

# 테스트넷 실행 (대시보드 포트 3001 자동 시작)
./target/release/scalping-bot --config config/testnet.toml

# 서브커맨드 없이도 동일 (하위 호환)
./target/release/scalping-bot trade --config config/testnet.toml

# 백테스트 실행 (scalping 전략)
./target/release/scalping-bot backtest --config config/default.toml \
  --start 2025-01-01 --end 2025-02-01 \
  --fee-rate 0.1 --output backtest_result.csv

# 백테스트 실행 (mean_reversion 전략)
./target/release/scalping-bot backtest --config config/mean_reversion.toml \
  --start 2025-01-01 --end 2025-02-01 \
  --fee-rate 0.1 --output backtest_mr.csv

# Docker
docker compose up -d

# 프론트엔드 (별도 터미널, 포트 3000)
cd dashboard && npm install && npm run dev
```

## 주요 의존성

| 크레이트 | 용도 |
|---------|------|
| `tokio` | 비동기 런타임 |
| `reqwest` | REST API |
| `tokio-tungstenite` | WebSocket |
| `ta` (0.5.0) | EMA, RSI, BollingerBands |
| `hmac` + `sha2` | HMAC-SHA256 서명 |
| `clap` | CLI |
| `tracing` | 로깅 |
| `axum` (0.7) | 대시보드 REST API + WebSocket |
| `tower-http` (0.6) | CORS, 트레이싱 미들웨어 |
| `teloxide` (0.13) | 텔레그램 봇 |

## 알려진 제한사항 / TODO

- [ ] dry-run 모드 구현 (`--dry-run` 플래그 파싱은 있으나 주문 실행에서 미분기)
- [ ] 테스트넷 실제 실행 검증 필요
- [ ] WebSocket 재연결 시 지표 상태 보존 검증
- [ ] 멀티 심볼 지원 없음 (단일 페어만)
- [x] 백테스트 기능 (`backtest` 서브커맨드로 구현됨)
- [ ] Rate limiting 구현 미비 (요청 가중치 관리)
- [ ] 대시보드 인증 없음 (프로덕션 사용 시 추가 필요)

## 코드 컨벤션

- `cargo fmt` 기본 설정 준수
- `cargo clippy` 경고 0개 유지
- API 모델 타입은 `#[allow(dead_code)]` 허용 (미사용 필드는 API 호환용)
- 에러 처리: `anyhow::Result` 사용
- SharedState write lock은 엔진에서만 잡음 (예외: 텔레그램 /start_bot, /stop_bot)
