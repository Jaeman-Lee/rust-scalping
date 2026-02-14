# Binance Scalping Bot

Rust로 구현한 Binance 암호화폐 초단타(스캘핑) 자동매매 봇입니다.
웹 대시보드와 텔레그램 봇을 통해 실시간 모니터링 및 원격 제어가 가능합니다.

## 기능

- **실시간 시세 수신**: WebSocket을 통한 Binance 캔들(kline) 데이터 스트리밍
- **기술지표 기반 매매**: EMA, RSI, 볼린저밴드를 활용한 스캘핑 전략
- **리스크 관리**: 손절/익절, 일일 거래 제한, 연속 손실 차단
- **웹 대시보드**: Axum REST API + Next.js 프론트엔드 (실시간 차트, 지표, 포지션)
- **텔레그램 봇**: 상태 조회, 매매 알림, 원격 일시정지/재개
- **거래 로깅**: CSV 형태의 거래 기록
- **Docker 지원**: 멀티스테이지 빌드로 어떤 환경에서든 실행 가능

## 빠른 시작

### 사전 준비

- Rust 1.70+
- Binance API 키
- (선택) Node.js 20+ - 프론트엔드 대시보드
- (선택) Telegram Bot Token - 텔레그램 알림

### 설치 및 실행

```bash
# 클론
git clone git@github.com:beautifulNH/rust-scalping.git
cd rust-scalping

# 환경변수 설정
cp .env.example .env
# .env 파일에 API 키 입력

# 빌드 및 테스트넷 실행
cargo build --release
./target/release/scalping-bot --config config/testnet.toml
```

### Docker로 실행

```bash
cp .env.example .env
# .env 파일에 API 키 입력

docker compose up -d
```

## 웹 대시보드

봇 실행 시 포트 3001에서 REST API + WebSocket 서버가 자동 시작됩니다.

### API 엔드포인트

| 엔드포인트 | 설명 |
|-----------|------|
| `GET /api/status` | 전체 상태 (가격, 지표, 포지션, 리스크) |
| `GET /api/trades?limit=50` | 최근 거래 내역 |
| `GET /api/indicators` | 현재 기술지표 값 |
| `GET /api/balance` | 잔고, 일일 PnL, 승률 |
| `GET /api/ws` | WebSocket 실시간 이벤트 스트림 |

### Next.js 프론트엔드

```bash
cd dashboard
npm install
npm run dev
# → http://localhost:3000
```

| 페이지 | 설명 |
|--------|------|
| `/` | 메인 대시보드 (차트, 포지션, 지표, 리스크, 거래 내역) |
| `/trades` | 거래 내역 전체 |
| `/settings` | 현재 설정값 조회 |

### 대시보드 설정

```toml
# config/testnet.toml
[dashboard]
enabled = true
port = 3001
host = "0.0.0.0"
```

## 텔레그램 봇

### 설정 방법

1. [@BotFather](https://t.me/BotFather)에서 봇 생성 → 토큰 발급
2. 봇에 `/start` 전송 후 Chat ID 확인
3. `.env`에 추가:
   ```
   TELEGRAM_BOT_TOKEN=your_token
   TELEGRAM_CHAT_ID=your_chat_id
   ```
4. `config/testnet.toml`에서 활성화:
   ```toml
   [telegram]
   enabled = true
   ```

### 명령어

| 명령어 | 기능 |
|--------|------|
| `/status` | 봇 상태, 현재 가격, 포지션, 지표 |
| `/balance` | 잔고, 일일 PnL, 거래 수, 승률 |
| `/trades` | 최근 5건 거래 내역 |
| `/pnl` | 일일 PnL 요약, 승률 |
| `/start_bot` | 매매 재개 |
| `/stop_bot` | 매매 일시정지 |
| `/config` | 현재 설정값 조회 |

### 자동 알림

- 매수/매도 체결 시 알림
- 리스크 한도 도달 경고
- 엔진 상태 변경 알림

## 설정

`config/default.toml` 파일에서 매매 파라미터를 조정할 수 있습니다:

| 설정 | 설명 | 기본값 |
|------|------|--------|
| `strategy.symbol` | 거래 심볼 | BTCUSDT |
| `strategy.ema_short` | 단기 EMA 기간 | 9 |
| `strategy.ema_long` | 장기 EMA 기간 | 21 |
| `strategy.rsi_period` | RSI 기간 | 14 |
| `trading.quantity` | 주문 수량 | 0.001 |
| `trading.stop_loss_pct` | 손절 비율(%) | 0.3 |
| `trading.take_profit_pct` | 익절 비율(%) | 0.5 |
| `trading.max_daily_trades` | 일일 최대 거래 횟수 | 100 |
| `dashboard.enabled` | 대시보드 활성화 | true |
| `dashboard.port` | 대시보드 포트 | 3001 |
| `telegram.enabled` | 텔레그램 봇 활성화 | false |

## 매매 전략

### 매수 조건 (3가지 모두 충족)
1. EMA(9)가 EMA(21) 위로 크로스
2. RSI < 70 (과매수 아님)
3. 가격이 볼린저밴드 하단 근처 또는 중간선 위로 돌파

### 매도 조건 (하나라도 충족)
1. EMA(9)가 EMA(21) 아래로 크로스
2. RSI > 70 (과매수)
3. 가격이 볼린저밴드 상단 도달
4. 손절가 도달 (-0.3%)
5. 익절가 도달 (+0.5%)

## 아키텍처

```
TradingEngine ──writes──> SharedState <──reads── Axum REST API / WebSocket
      │                                          Telegram commands
      └──sends──> broadcast<Event> ──subscribes──> WebSocket clients
                                                   Telegram alerts
```

## CLI 옵션

```
Options:
  -c, --config <CONFIG>  설정 파일 경로 [default: config/default.toml]
      --dry-run          시뮬레이션 모드 (실제 주문 없음)
  -h, --help             도움말
```

## 주의사항

- 이 봇은 교육 및 연구 목적으로 제작되었습니다
- 실제 자금으로 거래 시 손실이 발생할 수 있습니다
- 반드시 테스트넷에서 충분한 테스트 후 사용하세요
- API 키는 절대 공개 저장소에 커밋하지 마세요
- 대시보드에 인증이 없으므로 외부 노출 시 주의하세요
