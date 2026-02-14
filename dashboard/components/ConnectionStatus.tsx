"use client";

interface Props {
  isConnected: boolean;
}

export default function ConnectionStatus({ isConnected }: Props) {
  return (
    <div className="flex items-center gap-2">
      <div
        className={`w-2.5 h-2.5 rounded-full ${
          isConnected ? "bg-green-500" : "bg-red-500"
        }`}
      />
      <span className="text-sm text-gray-400">
        {isConnected ? "Connected" : "Disconnected"}
      </span>
    </div>
  );
}
