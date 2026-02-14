# 프로젝트 진행 기록

## 2026-02-14 - 웹 대시보드 + 텔레그램 봇 구현 & 테스트넷 검증

### 완료된 작업

1. **Shared State 인프라 (Phase 1)**
   - `src/dashboard/state.rs`: EngineState, DashboardEvent, 스냅샷 타입들
   - `src/dashboard/mod.rs`: SharedState (`Arc<RwLock<EngineState>>`), EventSender 타입
   - `src/trading/engine.rs`: SharedState/EventSender 통합, `update_shared_state()`, `is_paused` 지원
   - `src/trading/risk.rs`: `consecutive_losses()`, `account_balance()` 등 getter 추가
   - `src/config.rs`: DashboardConfig, TelegramConfig 추가 (`#[serde(default)]`)
   - `Cargo.toml`: axum 0.7, tower-http 0.6, teloxide 0.13 추가

2. **Axum 대시보드 API (Phase 2)**
   - `src/dashboard/server.rs`: Axum 서버 (CORS, graceful shutdown)
   - `src/dashboard/handlers.rs`: 5개 REST 엔드포인트 + WebSocket 핸들러
   - 엔드포인트: `/api/status`, `/api/trades`, `/api/indicators`, `/api/balance`, `/api/ws`

3. **텔레그램 봇 코드 (Phase 3)**
   - `src/telegram/bot.rs`: teloxide 디스패처 + alert listener spawn
   - `src/telegram/commands.rs`: 7개 명령어 (/status, /balance, /trades, /pnl, /start_bot, /stop_bot, /config)
   - `src/telegram/alerts.rs`: DashboardEvent 구독 → 자동 알림 전송

4. **Next.js 프론트엔드 (Phase 4)**
   - `dashboard/`: Next.js 14 + TypeScript + Tailwind 프로젝트
   - 컴포넌트: PriceChart, PositionCard, DailyStats, IndicatorPanel, RiskStatus, TradeTable, ConnectionStatus
   - Hooks: useWebSocket (자동 재연결), useApi (REST 폴링)
   - 3개 페이지: 메인 대시보드, 거래 내역, 설정

5. **통합 & 설정 (Phase 5)**
   - `config/default.toml`, `config/testnet.toml`: `[dashboard]`, `[telegram]` 섹션 추가
   - `.env.example`: TELEGRAM_BOT_TOKEN, TELEGRAM_CHAT_ID 추가
   - `docker-compose.yml`: 포트 3001 노출, 텔레그램 환경변수 추가
   - `CLAUDE.md` 전체 업데이트
   - `docs/DASHBOARD_USER_GUIDE.md`, `docs/DASHBOARD_AGENT_GUIDE.md` 작성

### 테스트넷 실행 검증 (2026-02-14)

| 항목 | 결과 |
|------|------|
| Binance 테스트넷 연결 | OK |
| WebSocket kline 스트림 | OK |
| 히스토리 워밍업 (100캔들) | OK |
| USDT 잔고 조회 | OK (10,000 USDT) |
| 대시보드 API (curl) | OK (latency 0ms) |
| Next.js 빌드 | OK (6 pages) |
| Next.js dev 서버 | OK (포트 3000) |
| API 프록시 (3000→3001) | OK |
| `cargo test` | 41/41 통과 |
| `cargo clippy` | 경고 0개 |

---

## 2026-02-12 - 초기 구현 완료

### 완료된 작업

1. **프로젝트 초기화**
   - Cargo.toml 의존성 설정 (17개 크레이트)
   - .gitignore, .env.example 생성

2. **설정 모듈** (`config.rs`)
   - TOML 파일 파싱 (exchange, strategy, trading, logging 섹션)
   - 환경변수에서 API 키 로드

3. **거래소 모듈** (`exchange/`)
   - HMAC-SHA256 서명 (`auth.rs`)
   - REST API 클라이언트 - 주문/잔고/캔들 조회 (`client.rs`)
   - WebSocket kline 스트림 + 자동 재연결 (`websocket.rs`)
   - API 요청/응답 타입 정의 (`models.rs`)

4. **기술지표** (`indicators/calculator.rs`)
   - EMA(9), EMA(21), RSI(14), 볼린저밴드(20, 2σ)
   - 이전 EMA 값 추적 (크로스오버 감지용)
   - warm_up() 메서드로 과거 데이터 사전 로드

5. **스캘핑 전략** (`strategy/`)
   - 매수 조건 3가지 (EMA 크로스, RSI, BB)
   - 매도 조건 5가지 (손절, 익절, EMA 크로스다운, RSI, BB상단)

6. **매매 엔진** (`trading/`)
   - engine.rs: WebSocket→지표→시그널→주문 파이프라인
   - orders.rs: 시장가/지정가 주문 래퍼
   - position.rs: 포지션 PnL 추적
   - risk.rs: 일일 거래한도, 손실한도, 연속손실, 포지션 크기 제한

7. **유틸리티**
   - CSV 거래 로깅 (`utils/logger.rs`)
   - tracing 기반 구조화된 로깅

8. **DevOps**
   - Dockerfile (멀티스테이지 빌드)
   - docker-compose.yml
   - GitHub Actions CI (fmt, clippy, test, build)
   - README.md (한국어)

9. **테스트** (41개 전체 통과)
   - config: 4개 (로드, 값 검증)
   - auth: 6개 (서명, 쿼리 빌드)
   - indicators: 8개 (EMA, RSI, BB, warm-up)
   - strategy: 10개 (매수/매도 조건 전체 커버)
   - position: 5개 (PnL 계산)
   - risk: 8개 (한도 체크, 리셋)

10. **GitHub 푸시** → `git@github.com:beautifulNH/rust-scalping.git`

---

## 다음 단계

### 즉시 진행 가능
- [ ] **텔레그램 봇 실제 연동**: BotFather 토큰 발급 → .env 설정 → E2E 검증
- [ ] `--dry-run` 모드 주문 분기 처리

### 중기
- [ ] 텔레그램 Chat ID 인증 (허용된 사용자만 명령)
- [ ] 텔레그램 알림 레벨 설정
- [ ] 대시보드 인증 (프로덕션용)
- [ ] 백테스트 기능

### 장기
- [ ] 멀티 심볼 지원
- [ ] Rate limiting 구현
- [ ] 수익률 차트 (일별/주별 히스토리)
