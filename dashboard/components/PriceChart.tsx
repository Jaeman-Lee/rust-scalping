"use client";

import { useEffect, useRef } from "react";
import type { DashboardEvent } from "@/hooks/useWebSocket";

interface Props {
  lastEvent: DashboardEvent | null;
}

interface PricePoint {
  time: number;
  price: number;
  ema_short?: number;
  ema_long?: number;
  bb_upper?: number;
  bb_lower?: number;
}

const MAX_POINTS = 200;

export default function PriceChart({ lastEvent }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const dataRef = useRef<PricePoint[]>([]);

  useEffect(() => {
    if (
      lastEvent?.type === "PriceUpdate" &&
      lastEvent.price &&
      lastEvent.price > 0
    ) {
      const point: PricePoint = {
        time: Date.now(),
        price: lastEvent.price,
        ema_short: lastEvent.indicators?.ema_short,
        ema_long: lastEvent.indicators?.ema_long,
        bb_upper: lastEvent.indicators?.bb_upper,
        bb_lower: lastEvent.indicators?.bb_lower,
      };

      dataRef.current.push(point);
      if (dataRef.current.length > MAX_POINTS) {
        dataRef.current.shift();
      }

      drawChart();
    }
  }, [lastEvent]);

  function drawChart() {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const data = dataRef.current;
    if (data.length < 2) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);
    const w = rect.width;
    const h = rect.height;

    ctx.clearRect(0, 0, w, h);

    // Compute bounds
    let min = Infinity;
    let max = -Infinity;
    for (const d of data) {
      const vals = [
        d.price,
        d.bb_upper,
        d.bb_lower,
        d.ema_short,
        d.ema_long,
      ].filter((v) => v !== undefined) as number[];
      for (const v of vals) {
        if (v < min) min = v;
        if (v > max) max = v;
      }
    }
    const padding = (max - min) * 0.05 || 1;
    min -= padding;
    max += padding;

    const xStep = w / (data.length - 1);
    const yScale = (v: number) => h - ((v - min) / (max - min)) * h;

    // Draw BB band fill
    ctx.fillStyle = "rgba(59, 130, 246, 0.08)";
    ctx.beginPath();
    for (let i = 0; i < data.length; i++) {
      const x = i * xStep;
      const y = yScale(data[i].bb_upper ?? data[i].price);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    for (let i = data.length - 1; i >= 0; i--) {
      const x = i * xStep;
      const y = yScale(data[i].bb_lower ?? data[i].price);
      ctx.lineTo(x, y);
    }
    ctx.closePath();
    ctx.fill();

    // Helper to draw a line
    function drawLine(
      getter: (d: PricePoint) => number | undefined,
      color: string,
      width: number = 1
    ) {
      ctx!.strokeStyle = color;
      ctx!.lineWidth = width;
      ctx!.beginPath();
      let started = false;
      for (let i = 0; i < data.length; i++) {
        const v = getter(data[i]);
        if (v === undefined) continue;
        const x = i * xStep;
        const y = yScale(v);
        if (!started) {
          ctx!.moveTo(x, y);
          started = true;
        } else {
          ctx!.lineTo(x, y);
        }
      }
      ctx!.stroke();
    }

    // Draw lines
    drawLine((d) => d.bb_upper, "rgba(59,130,246,0.3)");
    drawLine((d) => d.bb_lower, "rgba(59,130,246,0.3)");
    drawLine((d) => d.ema_long, "rgba(251,191,36,0.6)");
    drawLine((d) => d.ema_short, "rgba(34,197,94,0.6)");
    drawLine((d) => d.price, "#ffffff", 2);

    // Current price label
    const last = data[data.length - 1];
    ctx.fillStyle = "#ffffff";
    ctx.font = "bold 12px monospace";
    ctx.textAlign = "right";
    ctx.fillText(`$${last.price.toFixed(2)}`, w - 4, yScale(last.price) - 6);
  }

  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <h3 className="text-sm font-medium text-gray-400 mb-3">Price Chart</h3>
      <canvas
        ref={canvasRef}
        className="w-full h-64"
        style={{ display: "block" }}
      />
      <div className="flex gap-4 mt-2 text-xs text-gray-500">
        <span className="flex items-center gap-1">
          <span className="inline-block w-3 h-0.5 bg-white" /> Price
        </span>
        <span className="flex items-center gap-1">
          <span className="inline-block w-3 h-0.5 bg-green-500" /> EMA(9)
        </span>
        <span className="flex items-center gap-1">
          <span className="inline-block w-3 h-0.5 bg-yellow-500" /> EMA(21)
        </span>
        <span className="flex items-center gap-1">
          <span className="inline-block w-3 h-0.5 bg-blue-500 opacity-30" />{" "}
          BB
        </span>
      </div>
    </div>
  );
}
