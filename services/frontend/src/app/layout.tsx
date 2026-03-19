import { GeistMono } from "geist/font/mono";
import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "RealEstate Intelligence",
  description: "不動産投資意思決定プラットフォーム",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="ja" className={GeistMono.variable}>
      <body>{children}</body>
    </html>
  );
}
