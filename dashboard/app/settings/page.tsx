"use client";

import { useApi } from "@/hooks/useApi";

interface StatusResponse {
  current_price: number;
  symbol: string;
  risk: {
    max_daily_trades: number;
    max_daily_loss_pct: number;
  };
  is_running: boolean;
  is_paused: boolean;
}

export default function SettingsPage() {
  const { data, loading, error } = useApi<StatusResponse>("/api/status", 10000);

  if (loading) return <p className="text-gray-400">Loading...</p>;
  if (error) return <p className="text-red-400">Error: {error}</p>;
  if (!data) return null;

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Settings</h2>
      <div className="bg-gray-800 rounded-lg p-6 max-w-lg">
        <h3 className="text-sm font-medium text-gray-400 mb-4">
          Current Configuration
        </h3>
        <div className="space-y-3">
          <Row label="Symbol" value={data.symbol} />
          <Row label="Max Daily Trades" value={String(data.risk.max_daily_trades)} />
          <Row
            label="Max Daily Loss %"
            value={`${data.risk.max_daily_loss_pct}%`}
          />
          <Row label="Engine Running" value={data.is_running ? "Yes" : "No"} />
          <Row label="Trading Paused" value={data.is_paused ? "Yes" : "No"} />
        </div>
        <p className="text-xs text-gray-500 mt-6">
          Configuration is read-only. Edit config/testnet.toml or config/default.toml to change settings.
          Use Telegram /stop_bot and /start_bot to pause/resume trading.
        </p>
      </div>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between">
      <span className="text-gray-400">{label}</span>
      <span className="text-white font-mono">{value}</span>
    </div>
  );
}
