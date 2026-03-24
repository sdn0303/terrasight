import { render, screen, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";

// ─── Mocks ───────────────────────────────────────────

const mockUseStats = vi.fn();
vi.mock("@/features/stats/api/use-stats", () => ({
  useStats: (...args: unknown[]) => mockUseStats(...args),
}));

const mockUseMediaQuery = vi.fn();
vi.mock("@/hooks/use-media-query", () => ({
  useMediaQuery: (query: string) => mockUseMediaQuery(query),
}));

// ─── Fixtures ────────────────────────────────────────

const TEST_BBOX = { south: 35.65, west: 139.70, north: 35.70, east: 139.80 };

const STATS = {
  land_price: {
    avg_per_sqm: 850000,
    median_per_sqm: 720000,
    min_per_sqm: 320000,
    max_per_sqm: 3200000,
    count: 45,
  },
  risk: {
    flood_area_ratio: 0.15,
    steep_slope_area_ratio: 0.02,
    composite_risk: 0.18,
  },
  facilities: { schools: 12, medical: 28 },
  zoning_distribution: { 商業地域: 0.35 },
};

// ─── Tests ───────────────────────────────────────────

describe("DashboardStats", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders loading skeletons on desktop", async () => {
    mockUseStats.mockReturnValue({ data: undefined, isLoading: true });
    mockUseMediaQuery.mockImplementation((q: string) => {
      if (q === "(min-width: 768px)") return true; // tablet+
      if (q === "(min-width: 1280px)") return true; // desktop
      return false;
    });

    const { DashboardStats } = await import("@/components/dashboard-stats");
    render(<DashboardStats bbox={TEST_BBOX} zoom={12} />);

    expect(screen.getByLabelText("Area statistics loading")).toBeInTheDocument();
  });

  it("renders stat cards with data on desktop", async () => {
    mockUseStats.mockReturnValue({ data: STATS, isLoading: false });
    mockUseMediaQuery.mockImplementation((q: string) => {
      if (q === "(min-width: 768px)") return true;
      if (q === "(min-width: 1280px)") return true;
      return false;
    });

    const { DashboardStats } = await import("@/components/dashboard-stats");
    render(<DashboardStats bbox={TEST_BBOX} zoom={12} />);

    const region = screen.getByRole("region", { name: "Area statistics" });
    expect(region).toBeInTheDocument();
    expect(screen.getByText("AVG PRICE")).toBeInTheDocument();
    expect(screen.getByText("LISTINGS")).toBeInTheDocument();
    expect(screen.getByText("RISK")).toBeInTheDocument();
    expect(screen.getByText("FACILITIES")).toBeInTheDocument();
  });

  it("displays correct risk percentage", async () => {
    mockUseStats.mockReturnValue({ data: STATS, isLoading: false });
    mockUseMediaQuery.mockImplementation((q: string) => {
      if (q === "(min-width: 768px)") return true;
      if (q === "(min-width: 1280px)") return true;
      return false;
    });

    const { DashboardStats } = await import("@/components/dashboard-stats");
    render(<DashboardStats bbox={TEST_BBOX} zoom={12} />);

    // composite_risk = 0.18 → 18%
    expect(screen.getByText("18%")).toBeInTheDocument();
  });

  it("uses danger color when risk > 30%", async () => {
    const highRiskStats = {
      ...STATS,
      risk: { ...STATS.risk, composite_risk: 0.45 },
    };
    mockUseStats.mockReturnValue({ data: highRiskStats, isLoading: false });
    mockUseMediaQuery.mockImplementation((q: string) => {
      if (q === "(min-width: 768px)") return true;
      if (q === "(min-width: 1280px)") return true;
      return false;
    });

    const { DashboardStats } = await import("@/components/dashboard-stats");
    render(<DashboardStats bbox={TEST_BBOX} zoom={12} />);

    // 45% should render with danger color
    const riskValue = screen.getByText("45%");
    expect(riskValue).toBeInTheDocument();
    expect(riskValue.style.color).toBe("var(--accent-danger)");
  });

  it("returns null when no stats and not loading", async () => {
    mockUseStats.mockReturnValue({ data: undefined, isLoading: false });
    mockUseMediaQuery.mockImplementation((q: string) => {
      if (q === "(min-width: 768px)") return true;
      if (q === "(min-width: 1280px)") return true;
      return false;
    });

    const { DashboardStats } = await import("@/components/dashboard-stats");
    const { container } = render(<DashboardStats bbox={TEST_BBOX} zoom={12} />);

    expect(container.innerHTML).toBe("");
  });

  it("shows toggle button on mobile and reveals stats on tap", async () => {
    mockUseStats.mockReturnValue({ data: STATS, isLoading: false });
    mockUseMediaQuery.mockImplementation((q: string) => {
      // mobile: all breakpoints false
      if (q === "(min-width: 768px)") return false;
      if (q === "(min-width: 1280px)") return false;
      return false;
    });

    const { DashboardStats } = await import("@/components/dashboard-stats");
    render(<DashboardStats bbox={TEST_BBOX} zoom={12} />);

    const toggleBtn = screen.getByLabelText("Show area statistics");
    expect(toggleBtn).toBeInTheDocument();
    expect(screen.getByText("SHOW STATS")).toBeInTheDocument();

    fireEvent.click(toggleBtn);

    expect(screen.getByText("HIDE STATS")).toBeInTheDocument();
    expect(screen.getByText("AVG PRICE")).toBeInTheDocument();
  });

  it("sums facilities count correctly", async () => {
    mockUseStats.mockReturnValue({ data: STATS, isLoading: false });
    mockUseMediaQuery.mockImplementation((q: string) => {
      if (q === "(min-width: 768px)") return true;
      if (q === "(min-width: 1280px)") return true;
      return false;
    });

    const { DashboardStats } = await import("@/components/dashboard-stats");
    render(<DashboardStats bbox={TEST_BBOX} zoom={12} />);

    // 12 schools + 28 medical = 40
    expect(screen.getByText("40")).toBeInTheDocument();
    expect(screen.getByText("12 schools, 28 medical")).toBeInTheDocument();
  });
});
