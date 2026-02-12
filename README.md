# Binance Scalping Bot

Rust로 구현한 Binance 암호화폐 초단타(스캘핑) 자동매매 봇입니다.

## 기능

- **실시간 시세 수신**: WebSocket을 통한 Binance 캔들(kline) 데이터 스트리밍
- **기술지표 기반 매매**: EMA, RSI, 볼린저밴드를 활용한 스캘핑 전략
- **리스크 관리**: 손절/익절, 일일 거래 제한, 연속 손실 차단
- **거래 로깅**: CSV 형태의 거래 기록
- **Docker 지원**: 멀티스테이지 빌드로 어떤 환경에서든 실행 가능

## 빠른 시작

### 사전 준비

- Rust 1.70+
- Binance API 키

### 설치 및 실행

```bash
# 클론
git clone <repository-url>
cd scalping-bot

# 환경변수 설정
cp .env.example .env
# .env 파일에 API 키 입력

# 빌드 및 실행
cargo build --release
./target/release/scalping-bot --config config/default.toml
```

### Docker로 실행

```bash
cp .env.example .env
# .env 파일에 API 키 입력

docker compose up -d
```

### 테스트넷 실행

```bash
./target/release/scalping-bot --config config/testnet.toml
```

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
