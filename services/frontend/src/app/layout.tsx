import { GeistMono } from "geist/font/mono";
import type { Metadata } from "next";
import { Plus_Jakarta_Sans } from "next/font/google";
import { NextIntlClientProvider } from "next-intl";
import { NuqsAdapter } from "nuqs/adapters/next/app";
import { Suspense } from "react";
import { Providers } from "@/components/providers";
import { TooltipProvider } from "@/components/ui/tooltip";
import messages from "@/i18n/locales/ja.json";
import "./globals.css";

const jakarta = Plus_Jakarta_Sans({
  subsets: ["latin"],
  variable: "--font-sans",
  display: "swap",
});

export const metadata: Metadata = {
  title: "Terrasight",
  description: "不動産投資意思決定プラットフォーム",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html
      lang="ja"
      className={`dark ${jakarta.variable} ${GeistMono.variable}`}
    >
      <body>
        <NextIntlClientProvider locale="ja" messages={messages}>
          <NuqsAdapter>
            <Providers>
              <TooltipProvider>
                <Suspense>{children}</Suspense>
              </TooltipProvider>
            </Providers>
          </NuqsAdapter>
        </NextIntlClientProvider>
      </body>
    </html>
  );
}
