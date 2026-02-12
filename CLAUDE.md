# CLAUDE.md - Project Context for AI Agents

이 파일은 다른 Claude agent(또는 AI 도구)가 이 프로젝트를 즉시 이해하고 작업을 이어갈 수 있도록 작성되었습니다.

## 프로젝트 개요

- **이름**: rust-scalping (Binance 스캘핑 자동매매 봇)
- **언어**: Rust (edition 2021)
- **저장소**: git@github.com:beautifulNH/rust-scalping.git
- **상태**: 초기 구현 완료, 테스트넷 실행 전 단계

## 빌드 & 테스트

```bash
# 빌드 (Rust 1.70+ 필요)
cargo build --release

# 테스트 (41개)
cargo test

# 린트
cargo clippy

# 포맷
cargo fmt --check
```

## 프로젝트 구조

```
src/
├── main.rs              # 엔트리포인트 (CLI + 그레이스풀 셧다운)
├── config.rs            # TOML 설정 로드 + 환경변수
├── exchange/
│   ├── auth.rs          # HMAC-SHA256 서명 (Binance API 인증)
│   ├── client.rs        # REST API (주문/잔고/캔들 조회)
│   ├── models.rs        # API 요청/응답 타입 정의
│   └── websocket.rs     # WebSocket 실시간 kline 스트림
├── indicators/
│   └── calculator.rs    # 기술지표 래퍼 (EMA, RSI, 볼린저밴드)
├── strategy/
│   ├── scalping.rs      # 스캘핑 전략 (매수/매도 조건 평가)
│   └── signals.rs       # Signal enum (Buy/Sell/Hold)
├── trading/
│   ├── engine.rs        # 매매 엔진 (WebSocket→지표→시그널→주문)
│   ├── orders.rs        # 주문 관리 (시장가/지정가)
│   ├── position.rs      # 포지션 추적 (진입가, 수량, PnL)
│   └── risk.rs          # 리스크 관리 (일일한도, 연속손실, 포지션크기)
└── utils/
    └── logger.rs        # CSV 거래 기록 + tracing 초기화
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
```

## 매매 전략 로직

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

## 설정 파일

- `config/default.toml` - 실거래 설정 (api.binance.com)
- `config/testnet.toml` - 테스트넷 설정 (testnet.binance.vision)
- `.env` - API 키 (gitignore됨, `.env.example` 참고)

## 실행 방법

```bash
# .env 파일 필요
cp .env.example .env
# BINANCE_API_KEY, BINANCE_SECRET_KEY 입력

# 테스트넷 실행
./target/release/scalping-bot --config config/testnet.toml

# Docker
docker compose up -d
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

## 알려진 제한사항 / TODO

- [ ] dry-run 모드 구현 (`--dry-run` 플래그 파싱은 있으나 주문 실행에서 미분기)
- [ ] 테스트넷 실제 실행 검증 필요
- [ ] WebSocket 재연결 시 지표 상태 보존 검증
- [ ] 멀티 심볼 지원 없음 (단일 페어만)
- [ ] 백테스트 기능 없음
- [ ] Rate limiting 구현 미비 (요청 가중치 관리)

## 코드 컨벤션

- `cargo fmt` 기본 설정 준수
- `cargo clippy` 경고 0개 유지
- API 모델 타입은 `#[allow(dead_code)]` 허용 (미사용 필드는 API 호환용)
- 에러 처리: `anyhow::Result` 사용
