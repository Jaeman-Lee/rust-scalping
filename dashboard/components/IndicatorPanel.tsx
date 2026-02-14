"use client";

interface IndicatorSnapshot {
  ema_short: number;
  ema_long: number;
  rsi: number;
  bb_upper: number;
  bb_middle: number;
  bb_lower: number;
}

interface Props {
  indicators: IndicatorSnapshot | null;
}

export default function IndicatorPanel({ indicators }: Props) {
  if (!indicators) {
    return (
      <div className="bg-gray-800 rounded-lg p-4">
        <h3 className="text-sm font-medium text-gray-400 mb-3">Indicators</h3>
        <p className="text-gray-500 text-sm">Waiting for data...</p>
      </div>
    );
  }

  const rsiColor =
    indicators.rsi > 70
      ? "text-red-400"
      : indicators.rsi < 30
      ? "text-green-400"
      : "text-white";

  const emaTrend =
    indicators.ema_short > indicators.ema_long
      ? "text-green-400"
      : "text-red-400";

  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <h3 className="text-sm font-medium text-gray-400 mb-3">Indicators</h3>
      <div className="space-y-2">
        <div className="flex justify-between">
          <span className="text-gray-400">EMA(9)</span>
          <span className={`font-mono ${emaTrend}`}>
            {indicators.ema_short.toFixed(2)}
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-400">EMA(21)</span>
          <span className={`font-mono ${emaTrend}`}>
            {indicators.ema_long.toFixed(2)}
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-400">RSI</span>
          <span className={`font-mono font-bold ${rsiColor}`}>
            {indicators.rsi.toFixed(1)}
          </span>
        </div>
        <div className="border-t border-gray-700 pt-2 mt-2">
          <p className="text-xs text-gray-500 mb-1">Bollinger Bands</p>
          <div className="flex justify-between">
            <span className="text-gray-400">Upper</span>
            <span className="text-white font-mono">
              {indicators.bb_upper.toFixed(2)}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">Middle</span>
            <span className="text-white font-mono">
              {indicators.bb_middle.toFixed(2)}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">Lower</span>
            <span className="text-white font-mono">
              {indicators.bb_lower.toFixed(2)}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
