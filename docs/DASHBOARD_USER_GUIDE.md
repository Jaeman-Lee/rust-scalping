# Rust Scalping Bot - 사용자 가이드

> 최종 업데이트: 2026-02-20

---

## 프로젝트 현황

### 완료된 기능

| 기능 | 상태 | 비고 |
|------|------|------|
| 스캘핑 매매 엔진 | **완료** | EMA/RSI/BB 기반 자동매매 |
| Binance 테스트넷 연동 | **완료, 검증됨** | WebSocket + REST API |
| 웹 대시보드 API (Axum) | **완료, 검증됨** | 포트 3001, 5개 엔드포인트 |
| Next.js 프론트엔드 | **완료, 검증됨** | 포트 3000, 실시간 차트/지표 |
| 텔레그램 봇 코드 | **완료 (미연동)** | teloxide 기반, 실제 봇 토큰 연결 필요 |
| 리스크 관리 | **완료** | 일일한도, 연속손실, 포지션크기 |
| 백테스트 모듈 | **완료** | 과거 데이터 기반 전략 시뮬레이션 + 수수료 반영 |
| CI/CD | **완료** | GitHub Actions (fmt, clippy, test, build) |

### 테스트넷 실행 검증 결과 (2026-02-14)

```
Connected to Binance. Server time: 1771051166167
Warmed up with 100 historical candles
Trading engine started for BTCUSDT
Dashboard server starting on 0.0.0.0:3001
WebSocket connected
Initial USDT balance: 10000.00000000
```

| 확인 항목 | 결과 |
|-----------|------|
| Binance 테스트넷 연결 | OK |
| WebSocket kline 스트림 | OK |
| 히스토리 워밍업 (100캔들) | OK |
| 대시보드 API 응답 | OK (latency 0ms) |
| Next.js 렌더링 | OK |
| API 프록시 (3000→3001) | OK |

---

## 1. 실행 방법

### 사전 준비

```bash
# 1. .env 파일 설정
cp .env.example .env

# 2. 필수: Binance API 키 입력
# BINANCE_API_KEY=your_api_key_here
# BINANCE_SECRET_KEY=your_secret_key_here

# 3. 선택: 텔레그램 봇 (아래 "텔레그램 연동" 섹션 참고)
# TELEGRAM_BOT_TOKEN=your_bot_token_here
# TELEGRAM_CHAT_ID=your_chat_id_here
```

### 봇 실행

```bash
# 빌드
cargo build --release

# 테스트넷 실행 (대시보드 포트 3001 자동 시작)
./target/release/scalping-bot --config config/testnet.toml

# 실거래 실행 (주의!)
./target/release/scalping-bot --config config/default.toml
```

### 프론트엔드 실행 (별도 터미널)

```bash
cd dashboard
npm install
npm run dev
# → http://localhost:3000
```

### Docker 실행

```bash
docker compose up -d
```

---

## 2. 웹 대시보드

### 접속
- **프론트엔드**: `http://localhost:3000`
- **API 직접 호출**: `http://localhost:3001/api/status`

### 화면 구성

| 영역 | 설명 |
|------|------|
| **Price Chart** | 실시간 가격 + EMA(9/21) + Bollinger Bands 오버레이 |
| **Position Card** | 현재 포지션 진입가, 수량, 미실현 PnL |
| **Daily Stats** | 잔고, 일일 PnL, 승률, W/L 비율 |
| **Indicator Panel** | EMA, RSI, BB 현재 값 (색상으로 상태 표시) |
| **Risk Status** | 일일 거래한도, 손실한도, 연속손실 프로그레스바 |
| **Trade Table** | 최근 거래 내역 (시간, 방향, 가격, PnL) |

### 페이지

| 경로 | 설명 |
|------|------|
| `/` | 메인 대시보드 (전체 요약) |
| `/trades` | 거래 내역 전체 (최대 100건) |
| `/settings` | 현재 설정값 조회 (읽기 전용) |

### API 엔드포인트

| 엔드포인트 | 설명 |
|-----------|------|
| `GET /api/status` | 전체 상태 (가격, 지표, 포지션, 리스크) |
| `GET /api/trades?limit=50` | 최근 거래 내역 |
| `GET /api/indicators` | 현재 기술지표 값 |
| `GET /api/balance` | 잔고, 일일 PnL, 승률 |
| `GET /api/ws` | WebSocket (실시간 이벤트 스트림) |

### 설정

```toml
# config/testnet.toml 또는 config/default.toml
[dashboard]
enabled = true
port = 3001
host = "0.0.0.0"
```

---

## 3. 텔레그램 봇 (다음 연동 예정)

> **현재 상태**: Rust 코드 구현 완료. 실제 봇 토큰 연동 및 E2E 검증이 필요함.

### 연동 절차 (다음 단계)

#### Step 1: BotFather에서 봇 생성
1. 텔레그램에서 [@BotFather](https://t.me/BotFather) 대화 시작
2. `/newbot` 입력
3. 봇 이름 입력 (예: `My Scalping Bot`)
4. 봇 username 입력 (예: `my_scalping_bot`)
5. **발급된 토큰 복사** (예: `123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11`)

#### Step 2: Chat ID 확인
1. 생성된 봇에 `/start` 메시지 전송
2. 브라우저에서 `https://api.telegram.org/bot<YOUR_TOKEN>/getUpdates` 접속
3. 응답 JSON에서 `"chat":{"id": 123456789}` 확인
4. 또는 [@userinfobot](https://t.me/userinfobot)에 메시지 전송하여 ID 확인

#### Step 3: .env 설정
```bash
TELEGRAM_BOT_TOKEN=123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11
TELEGRAM_CHAT_ID=123456789
```

#### Step 4: config에서 텔레그램 활성화
```toml
[telegram]
enabled = true
```

#### Step 5: 봇 재시작 후 확인
```bash
./target/release/scalping-bot --config config/testnet.toml
# 텔레그램에서 /status 입력 → 응답 확인
```

### 명령어 (구현 완료, 연동 후 사용 가능)

| 명령어 | 기능 |
|--------|------|
| `/status` | 봇 상태, 현재 가격, 포지션, 지표 |
| `/balance` | 잔고, 일일 PnL, 거래 수, 승률 |
| `/trades` | 최근 5건 거래 내역 |
| `/pnl` | 일일 PnL 요약, 승률 |
| `/start_bot` | 매매 재개 (일시정지 해제) |
| `/stop_bot` | 매매 일시정지 |
| `/config` | 현재 전략/거래 설정값 |

### 자동 알림 (구현 완료, 연동 후 동작)

| 이벤트 | 알림 내용 |
|--------|----------|
| 매수 체결 | BUY 수량 @ 가격 |
| 매도 체결 | SELL 수량 @ 가격, PnL |
| 리스크 한도 도달 | Risk Alert 메시지 |
| 엔진 상태 변경 | Running/Paused 상태 |

---

## 4. 백테스트

> 과거 데이터를 기반으로 매매 전략의 수익성을 검증합니다. 수수료 반영, 일일 리셋, 리스크 관리 모두 라이브 엔진과 동일하게 동작합니다.

### 실행 방법

```bash
# 기본 실행 (최근 1개월, 수수료 0.1%, 초기 잔고 $10,000)
cargo run -- backtest --config config/default.toml \
  --start 2025-01-01 --end 2025-02-01

# 수수료율/초기잔고 커스텀 + CSV 출력
cargo run -- backtest --config config/default.toml \
  --start 2025-06-01 --end 2025-07-01 \
  --fee-rate 0.075 --initial-balance 50000 \
  --output backtest_result.csv

# 짧은 기간 테스트 (1주일)
cargo run -- backtest --config config/testnet.toml \
  --start 2025-12-01 --end 2025-12-08
```

### CLI 옵션

| 옵션 | 필수 | 기본값 | 설명 |
|------|------|--------|------|
| `--start` | O | - | 시작일 (YYYY-MM-DD) |
| `--end` | O | - | 종료일 (YYYY-MM-DD) |
| `--config` | X | `config/default.toml` | 설정 파일 (전략 파라미터 결정) |
| `--fee-rate` | X | `0.1` | 수수료율 (%, 예: 0.1 = 0.1%) |
| `--initial-balance` | X | `10000.0` | 시뮬레이션 초기 잔고 (USDT) |
| `--output` | X | - | 거래 상세 CSV 파일 경로 |

### 출력 예시

```
══════════════════════════════════════════
  BACKTEST RESULTS: BTCUSDT (1m)
  Period: 2025-01-01 ~ 2025-02-01
  Candles: 44,640
══════════════════════════════════════════
  Initial Balance:  $10000.00
  Final Balance:    $10487.00
  Total Return:     +4.87%
──────────────────────────────────────────
  Total Trades:     142
  Wins / Losses:    81 / 61
  Win Rate:         57.04%
  Avg Win:          +0.3200%
  Avg Loss:         -0.2100%
  Profit Factor:    1.44
  Max Drawdown:     -1.23%
  Sharpe Ratio:     1.82
  Total Fees:       $28.40
══════════════════════════════════════════
```

### CSV 출력 형식

`--output` 옵션 사용 시 각 거래의 상세 기록이 CSV로 저장됩니다:

```csv
entry_time,exit_time,entry_price,exit_price,quantity,pnl,pnl_pct,fee,reason
2025-01-03 14:22:00,2025-01-03 14:35:00,97250.00,97530.00,0.001000,0.2600,0.2674,0.1948,EMA cross up -> Take profit hit
```

### 참고사항

- Binance API 키가 필요합니다 (데이터 조회용, 주문 실행 없음)
- 캔들 종가를 체결가로 사용합니다 (슬리피지 미반영)
- 수수료는 매수/매도 양쪽 모두 적용됩니다 (왕복 수수료)
- 기간 종료 시 미청산 포지션은 강제 청산됩니다
- Binance rate limit 준수를 위해 데이터 수집 시 요청 간 200ms 딜레이가 있습니다

---

## 5. 매매 전략

### 매수 조건 (3가지 모두 충족)
1. EMA(9) > EMA(21) 크로스오버 (이전 캔들에서는 아래였음)
2. RSI < 70
3. 가격이 볼린저밴드 하단 30% 이내 또는 중간선 위

### 매도 조건 (하나라도 충족)
1. 손절: PnL <= -0.3%
2. 익절: PnL >= +0.5%
3. EMA(9) < EMA(21) 크로스다운
4. RSI > 70
5. 볼린저밴드 상단 5% 이내 도달

---

## 6. 알려진 제한사항

- `--dry-run` 모드 미구현 (플래그만 파싱됨)
- 멀티 심볼 미지원 (단일 페어만)
- Rate limiting 미구현
- 대시보드 인증 없음 (프로덕션 사용 시 추가 필요)
- 백테스트 시 슬리피지 미반영 (캔들 종가 = 체결가)
