import { NuqsAdapter } from "nuqs/adapters/react";
import { Suspense } from "react";
import { ErrorBoundary } from "@/components/error-boundary";
import { Providers } from "@/components/providers";
import { TooltipProvider } from "@/components/ui/tooltip";
import Home from "@/pages/Home";

export function App() {
  return (
    <ErrorBoundary>
      <NuqsAdapter>
        <Providers>
          <TooltipProvider>
            <Suspense fallback={null}>
              <Home />
            </Suspense>
          </TooltipProvider>
        </Providers>
      </NuqsAdapter>
    </ErrorBoundary>
  );
}
