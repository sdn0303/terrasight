import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { OpportunitiesSheet } from "@/components/opportunities/opportunities-sheet";

function wrap(ui: ReactNode) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>{ui}</QueryClientProvider>,
  );
}

describe("OpportunitiesSheet", () => {
  it("renders nothing when closed", () => {
    wrap(<OpportunitiesSheet open={false} onClose={vi.fn()} />);
    expect(screen.queryByText("Opportunities")).toBeNull();
  });

  it("renders title when open", () => {
    wrap(<OpportunitiesSheet open={true} onClose={vi.fn()} />);
    expect(screen.getByText("Opportunities")).toBeInTheDocument();
  });
});
