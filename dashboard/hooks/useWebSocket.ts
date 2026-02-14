import { useEffect, useRef, useState, useCallback } from "react";

export interface DashboardEvent {
  type: "PriceUpdate" | "TradeExecuted" | "RiskAlert" | "EngineStatusChanged";
  price?: number;
  symbol?: string;
  indicators?: IndicatorSnapshot;
  trade?: TradeSnapshot;
  message?: string;
  is_running?: boolean;
  is_paused?: boolean;
}

export interface IndicatorSnapshot {
  ema_short: number;
  ema_long: number;
  rsi: number;
  bb_upper: number;
  bb_middle: number;
  bb_lower: number;
}

export interface TradeSnapshot {
  side: string;
  entry_price: number;
  exit_price: number;
  quantity: number;
  pnl: number;
  pnl_pct: number;
  timestamp: string;
}

export function useWebSocket(url: string) {
  const [lastEvent, setLastEvent] = useState<DashboardEvent | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimerRef = useRef<NodeJS.Timeout | null>(null);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    const ws = new WebSocket(url);

    ws.onopen = () => {
      setIsConnected(true);
      if (reconnectTimerRef.current) {
        clearTimeout(reconnectTimerRef.current);
        reconnectTimerRef.current = null;
      }
    };

    ws.onmessage = (event) => {
      try {
        const data: DashboardEvent = JSON.parse(event.data);
        setLastEvent(data);
      } catch {
        // ignore parse errors
      }
    };

    ws.onclose = () => {
      setIsConnected(false);
      wsRef.current = null;
      reconnectTimerRef.current = setTimeout(connect, 3000);
    };

    ws.onerror = () => {
      ws.close();
    };

    wsRef.current = ws;
  }, [url]);

  useEffect(() => {
    connect();
    return () => {
      if (reconnectTimerRef.current) clearTimeout(reconnectTimerRef.current);
      wsRef.current?.close();
    };
  }, [connect]);

  return { lastEvent, isConnected };
}
