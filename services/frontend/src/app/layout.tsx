import { GeistMono } from "geist/font/mono";
import type { Metadata } from "next";
import { NuqsAdapter } from "nuqs/adapters/next/app";
import { Providers } from "@/components/providers";
import { TooltipProvider } from "@/components/ui/tooltip";
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
    <html lang="ja" className={`dark ${GeistMono.variable}`}>
      <body>
        <NuqsAdapter>
          <Providers>
            <TooltipProvider>{children}</TooltipProvider>
          </Providers>
        </NuqsAdapter>
      </body>
    </html>
  );
}
