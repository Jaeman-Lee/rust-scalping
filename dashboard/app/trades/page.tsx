"use client";

import { useApi } from "@/hooks/useApi";
import TradeTable from "@/components/TradeTable";

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

export default function TradesPage() {
  const { data, loading, error } = useApi<TradesResponse>(
    "/api/trades?limit=100",
    5000
  );

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Trade History</h2>
      {loading && <p className="text-gray-400">Loading...</p>}
      {error && <p className="text-red-400">Error: {error}</p>}
      {data && <TradeTable trades={data.trades} />}
    </div>
  );
}
