"use client";

interface PositionSnapshot {
  entry_price: number;
  quantity: number;
  entry_time: string;
  unrealized_pnl: number;
  unrealized_pnl_pct: number;
}

interface Props {
  position: PositionSnapshot | null;
  currentPrice: number;
}

export default function PositionCard({ position, currentPrice }: Props) {
  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <h3 className="text-sm font-medium text-gray-400 mb-3">Position</h3>
      {position ? (
        <div className="space-y-2">
          <div className="flex justify-between">
            <span className="text-gray-400">Entry</span>
            <span className="text-white font-mono">
              ${position.entry_price.toFixed(2)}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">Current</span>
            <span className="text-white font-mono">
              ${currentPrice.toFixed(2)}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">Quantity</span>
            <span className="text-white font-mono">
              {position.quantity.toFixed(6)}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">PnL</span>
            <span
              className={`font-mono font-bold ${
                position.unrealized_pnl >= 0
                  ? "text-green-400"
                  : "text-red-400"
              }`}
            >
              {position.unrealized_pnl >= 0 ? "+" : ""}
              {position.unrealized_pnl.toFixed(4)} (
              {position.unrealized_pnl_pct >= 0 ? "+" : ""}
              {position.unrealized_pnl_pct.toFixed(2)}%)
            </span>
          </div>
        </div>
      ) : (
        <p className="text-gray-500 text-sm">No open position</p>
      )}
    </div>
  );
}
