# 프로젝트 진행 기록

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

### 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo fmt --check` | ✅ 통과 |
| `cargo clippy` | ✅ 경고 0개 |
| `cargo test` | ✅ 41/41 통과 |
| `cargo build --release` | ✅ 컴파일 성공 |

### 다음 단계

- [ ] Binance 테스트넷 API 키 발급 → `.env` 설정 → 실행 테스트
- [ ] `--dry-run` 모드 실제 분기 처리 구현
- [ ] 백테스트 기능 추가
- [ ] 멀티 심볼 지원
- [ ] 수익률 대시보드 / 텔레그램 알림
