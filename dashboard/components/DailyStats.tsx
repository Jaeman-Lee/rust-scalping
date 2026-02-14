"use client";

interface Props {
  dailyPnl: number;
  dailyTrades: number;
  winRate: number;
  totalWins: number;
  totalLosses: number;
  accountBalance: number;
}

export default function DailyStats({
  dailyPnl,
  dailyTrades,
  winRate,
  totalWins,
  totalLosses,
  accountBalance,
}: Props) {
  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <h3 className="text-sm font-medium text-gray-400 mb-3">Daily Stats</h3>
      <div className="space-y-2">
        <div className="flex justify-between">
          <span className="text-gray-400">Balance</span>
          <span className="text-white font-mono">
            ${accountBalance.toFixed(2)}
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-400">Daily PnL</span>
          <span
            className={`font-mono font-bold ${
              dailyPnl >= 0 ? "text-green-400" : "text-red-400"
            }`}
          >
            {dailyPnl >= 0 ? "+" : ""}
            {dailyPnl.toFixed(4)}
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-400">Trades</span>
          <span className="text-white font-mono">{dailyTrades}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-400">Win Rate</span>
          <span className="text-white font-mono">{winRate.toFixed(1)}%</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-400">W / L</span>
          <span className="font-mono">
            <span className="text-green-400">{totalWins}</span>
            {" / "}
            <span className="text-red-400">{totalLosses}</span>
          </span>
        </div>
      </div>
    </div>
  );
}
