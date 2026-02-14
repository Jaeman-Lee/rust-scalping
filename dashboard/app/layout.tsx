import type { Metadata } from "next";
import "./globals.css";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Scalping Bot Dashboard",
  description: "Real-time monitoring for the Binance scalping bot",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="min-h-screen">
        <nav className="border-b border-gray-700 px-6 py-3">
          <div className="max-w-7xl mx-auto flex items-center justify-between">
            <h1 className="text-lg font-bold">Scalping Bot</h1>
            <div className="flex gap-4 text-sm">
              <Link
                href="/"
                className="text-gray-300 hover:text-white transition"
              >
                Dashboard
              </Link>
              <Link
                href="/trades"
                className="text-gray-300 hover:text-white transition"
              >
                Trades
              </Link>
              <Link
                href="/settings"
                className="text-gray-300 hover:text-white transition"
              >
                Settings
              </Link>
            </div>
          </div>
        </nav>
        <main className="max-w-7xl mx-auto px-6 py-6">{children}</main>
      </body>
    </html>
  );
}
