"use client";

import { useWebSocket } from "@/hooks/useWebSocket";
import { useApi } from "@/hooks/useApi";
import ConnectionStatus from "@/components/ConnectionStatus";
import PriceChart from "@/components/PriceChart";
import PositionCard from "@/components/PositionCard";
import DailyStats from "@/components/DailyStats";
import IndicatorPanel from "@/components/IndicatorPanel";
import RiskStatus from "@/components/RiskStatus";
import TradeTable from "@/components/TradeTable";

interface StatusResponse {
  current_price: number;
  symbol: string;
  indicators: {
    ema_short: number;
    ema_long: number;
    rsi: number;
    bb_upper: number;
    bb_middle: number;
    bb_lower: number;
  } | null;
  position: {
    entry_price: number;
    quantity: number;
    entry_time: string;
    unrealized_pnl: number;
    unrealized_pnl_pct: number;
  } | null;
  risk: {
    daily_trades: number;
    daily_pnl: number;
    consecutive_losses: number;
    account_balance: number;
    max_daily_trades: number;
    max_daily_loss_pct: number;
    total_wins: number;
    total_losses: number;
  };
  is_running: boolean;
  is_paused: boolean;
}

interface TradesResponse {
  trades: Array<{
    side: string;
    entry_price: number;
    exit_price: number;
    quantity: number;
    pnl: number;
    pnl_pct: number;
    timestamp: string;
  }>;
}

const WS_URL =
  typeof window !== "undefined"
    ? `ws://${window.location.hostname}:3001/api/ws`
    : "ws://localhost:3001/api/ws";

export default function Home() {
  const { lastEvent, isConnected } = useWebSocket(WS_URL);
  const { data: status } = useApi<StatusResponse>("/api/status", 3000);
  const { data: tradesData } = useApi<TradesResponse>("/api/trades?limit=10", 5000);

  const currentPrice = status?.current_price ?? 0;
  const risk = status?.risk;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">{status?.symbol ?? "---"}</h2>
          <p className="text-3xl font-mono font-bold mt-1">
            ${currentPrice.toFixed(2)}
          </p>
        </div>
        <div className="flex items-center gap-4">
          <ConnectionStatus isConnected={isConnected} />
          {status && (
            <div className="flex gap-2">
              <span
                className={`px-2 py-1 rounded text-xs font-medium ${
                  status.is_running
                    ? "bg-green-900 text-green-300"
                    : "bg-red-900 text-red-300"
                }`}
              >
                {status.is_running ? "Running" : "Stopped"}
              </span>
              {status.is_paused && (
                <span className="px-2 py-1 rounded text-xs font-medium bg-yellow-900 text-yellow-300">
                  Paused
                </span>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Chart */}
      <PriceChart lastEvent={lastEvent} />

      {/* Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <PositionCard
          position={status?.position ?? null}
          currentPrice={currentPrice}
        />
        <DailyStats
          dailyPnl={risk?.daily_pnl ?? 0}
          dailyTrades={risk?.daily_trades ?? 0}
          winRate={
            risk
              ? risk.total_wins + risk.total_losses > 0
                ? (risk.total_wins / (risk.total_wins + risk.total_losses)) * 100
                : 0
              : 0
          }
          totalWins={risk?.total_wins ?? 0}
          totalLosses={risk?.total_losses ?? 0}
          accountBalance={risk?.account_balance ?? 0}
        />
        <IndicatorPanel indicators={status?.indicators ?? null} />
        <RiskStatus
          dailyTrades={risk?.daily_trades ?? 0}
          maxDailyTrades={risk?.max_daily_trades ?? 100}
          dailyPnl={risk?.daily_pnl ?? 0}
          maxDailyLossPct={risk?.max_daily_loss_pct ?? 2}
          accountBalance={risk?.account_balance ?? 0}
          consecutiveLosses={risk?.consecutive_losses ?? 0}
        />
      </div>

      {/* Recent Trades */}
      <TradeTable trades={tradesData?.trades ?? []} />
    </div>
  );
}
