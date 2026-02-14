"use client";

interface Props {
  dailyTrades: number;
  maxDailyTrades: number;
  dailyPnl: number;
  maxDailyLossPct: number;
  accountBalance: number;
  consecutiveLosses: number;
}

export default function RiskStatus({
  dailyTrades,
  maxDailyTrades,
  dailyPnl,
  maxDailyLossPct,
  accountBalance,
  consecutiveLosses,
}: Props) {
  const tradeUsage = maxDailyTrades > 0 ? (dailyTrades / maxDailyTrades) * 100 : 0;
  const maxLoss = accountBalance * maxDailyLossPct / 100;
  const lossUsage = maxLoss > 0 ? (Math.abs(Math.min(dailyPnl, 0)) / maxLoss) * 100 : 0;
  const consLossUsage = (consecutiveLosses / 5) * 100;

  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <h3 className="text-sm font-medium text-gray-400 mb-3">Risk Status</h3>
      <div className="space-y-3">
        <ProgressBar
          label={`Trades: ${dailyTrades}/${maxDailyTrades}`}
          value={tradeUsage}
        />
        <ProgressBar
          label={`Daily Loss: ${Math.abs(Math.min(dailyPnl, 0)).toFixed(2)}/${maxLoss.toFixed(2)}`}
          value={lossUsage}
        />
        <ProgressBar
          label={`Consec. Losses: ${consecutiveLosses}/5`}
          value={consLossUsage}
        />
      </div>
    </div>
  );
}

function ProgressBar({ label, value }: { label: string; value: number }) {
  const clamped = Math.min(value, 100);
  const color =
    clamped < 50
      ? "bg-green-500"
      : clamped < 80
      ? "bg-yellow-500"
      : "bg-red-500";

  return (
    <div>
      <div className="flex justify-between text-xs mb-1">
        <span className="text-gray-400">{label}</span>
        <span className="text-gray-400">{clamped.toFixed(0)}%</span>
      </div>
      <div className="w-full bg-gray-700 rounded-full h-2">
        <div
          className={`${color} h-2 rounded-full transition-all`}
          style={{ width: `${clamped}%` }}
        />
      </div>
    </div>
  );
}
