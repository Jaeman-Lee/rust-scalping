"use client";

interface TradeSnapshot {
  side: string;
  entry_price: number;
  exit_price: number;
  quantity: number;
  pnl: number;
  pnl_pct: number;
  timestamp: string;
}

interface Props {
  trades: TradeSnapshot[];
}

export default function TradeTable({ trades }: Props) {
  if (trades.length === 0) {
    return (
      <div className="bg-gray-800 rounded-lg p-4">
        <h3 className="text-sm font-medium text-gray-400 mb-3">
          Recent Trades
        </h3>
        <p className="text-gray-500 text-sm">No trades yet</p>
      </div>
    );
  }

  return (
    <div className="bg-gray-800 rounded-lg p-4 overflow-x-auto">
      <h3 className="text-sm font-medium text-gray-400 mb-3">Recent Trades</h3>
      <table className="w-full text-sm">
        <thead>
          <tr className="text-gray-400 border-b border-gray-700">
            <th className="text-left py-2 pr-4">Time</th>
            <th className="text-left py-2 pr-4">Side</th>
            <th className="text-right py-2 pr-4">Entry</th>
            <th className="text-right py-2 pr-4">Exit</th>
            <th className="text-right py-2 pr-4">Qty</th>
            <th className="text-right py-2">PnL</th>
          </tr>
        </thead>
        <tbody>
          {trades.map((trade, i) => (
            <tr key={i} className="border-b border-gray-700/50">
              <td className="py-2 pr-4 text-gray-400 font-mono text-xs">
                {new Date(trade.timestamp).toLocaleTimeString()}
              </td>
              <td className="py-2 pr-4">
                <span
                  className={`font-bold ${
                    trade.side === "BUY" ? "text-green-400" : "text-red-400"
                  }`}
                >
                  {trade.side}
                </span>
              </td>
              <td className="py-2 pr-4 text-right font-mono text-white">
                {trade.entry_price.toFixed(2)}
              </td>
              <td className="py-2 pr-4 text-right font-mono text-white">
                {trade.exit_price > 0 ? trade.exit_price.toFixed(2) : "-"}
              </td>
              <td className="py-2 pr-4 text-right font-mono text-white">
                {trade.quantity.toFixed(6)}
              </td>
              <td
                className={`py-2 text-right font-mono font-bold ${
                  trade.pnl >= 0 ? "text-green-400" : "text-red-400"
                }`}
              >
                {trade.side === "SELL"
                  ? `${trade.pnl >= 0 ? "+" : ""}${trade.pnl.toFixed(4)} (${
                      trade.pnl_pct >= 0 ? "+" : ""
                    }${trade.pnl_pct.toFixed(2)}%)`
                  : "-"}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
