import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { InsightDrawer } from "@/components/layout/insight-drawer";
import type { DrawerTab } from "@/stores/ui-store";

const tabs: { id: DrawerTab; label: string; content: ReactNode }[] = [
  {
    id: "intel",
    label: "Intel",
    content: <div data-testid="intel-body">Intel body</div>,
  },
  {
    id: "trend",
    label: "Trend",
    content: <div data-testid="trend-body">Trend body</div>,
  },
  {
    id: "risk",
    label: "Risk",
    content: <div data-testid="risk-body">Risk body</div>,
  },
  {
    id: "infra",
    label: "Infra",
    content: <div data-testid="infra-body">Infra body</div>,
  },
];

describe("InsightDrawer", () => {
  const baseProps = {
    open: true,
    onClose: vi.fn(),
    title: "渋谷区恵比寿 1-2-3",
    subtitle: "商業 · 80% / 600%",
    tabs,
    activeTab: "intel" as DrawerTab,
    onTabChange: vi.fn(),
  };

  it("renders nothing when closed", () => {
    const { container } = render(<InsightDrawer {...baseProps} open={false} />);
    expect(container.querySelector('[role="complementary"]')).toBeNull();
  });

  it("renders title and subtitle", () => {
    render(<InsightDrawer {...baseProps} />);
    expect(screen.getByText("渋谷区恵比寿 1-2-3")).toBeInTheDocument();
    expect(screen.getByText("商業 · 80% / 600%")).toBeInTheDocument();
  });

  it("renders all tab buttons", () => {
    render(<InsightDrawer {...baseProps} />);
    expect(screen.getByRole("tab", { name: "Intel" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Trend" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Risk" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Infra" })).toBeInTheDocument();
  });

  it("renders only the active tab's content", () => {
    render(<InsightDrawer {...baseProps} activeTab="trend" />);
    expect(screen.getByTestId("trend-body")).toBeInTheDocument();
    expect(screen.queryByTestId("intel-body")).toBeNull();
    expect(screen.queryByTestId("risk-body")).toBeNull();
    expect(screen.queryByTestId("infra-body")).toBeNull();
  });

  it("marks the active tab with aria-selected=true", () => {
    render(<InsightDrawer {...baseProps} activeTab="risk" />);
    expect(screen.getByRole("tab", { name: "Risk" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByRole("tab", { name: "Intel" })).toHaveAttribute(
      "aria-selected",
      "false",
    );
  });

  it("clicking a tab calls onTabChange", async () => {
    const onTabChange = vi.fn();
    const user = userEvent.setup();
    render(<InsightDrawer {...baseProps} onTabChange={onTabChange} />);
    await user.click(screen.getByRole("tab", { name: "Risk" }));
    expect(onTabChange).toHaveBeenCalledWith("risk");
  });

  it("clicking close calls onClose", async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    render(<InsightDrawer {...baseProps} onClose={onClose} />);
    await user.click(screen.getByRole("button", { name: /close drawer/i }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("renders a selected badge in the header", () => {
    render(<InsightDrawer {...baseProps} />);
    expect(screen.getByText(/SELECTED/i)).toBeInTheDocument();
  });
});
