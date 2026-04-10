import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { LayerControlPanel } from "@/components/layer/layer-control-panel";
import { LayerToggleRow } from "@/components/layer/layer-toggle-row";
import { LAYERS } from "@/lib/layers";
import { useMapStore } from "@/stores/map-store";

// Later tasks in this phase will add ThemeCard / LayerControlPanel /
// ThemesPanel describe blocks to this same file. Keep this mock at the
// top-level so all suites pick it up.
vi.mock("next-intl", () => ({
  useTranslations: () => (key: string) => key,
}));

describe("LayerToggleRow", () => {
  const baseProps = {
    id: "flood",
    label: "浸水想定",
    swatch: "#3b82f6",
    checked: false,
    onToggle: vi.fn(),
  };

  it("renders label", () => {
    render(<LayerToggleRow {...baseProps} />);
    expect(screen.getByText("浸水想定")).toBeInTheDocument();
  });

  it("renders a swatch with the provided color", () => {
    const { container } = render(<LayerToggleRow {...baseProps} />);
    const swatch = container.querySelector(
      '[data-testid="swatch"]',
    ) as HTMLElement;
    expect(swatch).not.toBeNull();
    expect(swatch.style.background).toContain("rgb(59, 130, 246)");
  });

  it("omits the swatch when no color is provided", () => {
    const { container } = render(
      <LayerToggleRow {...baseProps} swatch={undefined} />,
    );
    expect(container.querySelector('[data-testid="swatch"]')).toBeNull();
  });

  it("renders a switch with checked state", () => {
    render(<LayerToggleRow {...baseProps} checked={true} />);
    const sw = screen.getByRole("switch");
    expect(sw.getAttribute("aria-checked")).toBe("true");
  });

  it("clicking the switch calls onToggle with id", async () => {
    const onToggle = vi.fn();
    const user = userEvent.setup();
    render(<LayerToggleRow {...baseProps} onToggle={onToggle} />);
    await user.click(screen.getByRole("switch"));
    expect(onToggle).toHaveBeenCalledWith("flood");
  });
});

const defaultVisibleLayers = new Set(
  LAYERS.filter((l) => l.defaultEnabled).map((l) => l.id),
);

describe("LayerControlPanel", () => {
  beforeEach(() => {
    useMapStore.setState({ visibleLayers: new Set(defaultVisibleLayers) });
  });

  it("renders nothing when closed", () => {
    const { container } = render(
      <LayerControlPanel open={false} onClose={vi.fn()} />,
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders the title and active count subtitle when open", () => {
    useMapStore.setState({ visibleLayers: new Set(["flood", "station"]) });
    render(<LayerControlPanel open={true} onClose={vi.fn()} />);
    expect(screen.getByText("Data Layers")).toBeInTheDocument();
    expect(screen.getByText("2 active")).toBeInTheDocument();
  });

  it("renders all 5 category section headers", () => {
    render(<LayerControlPanel open={true} onClose={vi.fn()} />);
    // english labels from CATEGORIES
    expect(screen.getByText(/HOW MUCH/)).toBeInTheDocument();
    expect(screen.getByText(/IS IT SAFE/)).toBeInTheDocument();
    expect(screen.getByText(/WHAT'S THE GROUND/)).toBeInTheDocument();
    expect(screen.getByText(/WHAT'S NEARBY/)).toBeInTheDocument();
    expect(screen.getByText(/WHERE AM I/)).toBeInTheDocument();
  });

  it("renders one toggle per layer in LAYERS", () => {
    render(<LayerControlPanel open={true} onClose={vi.fn()} />);
    const switches = screen.getAllByRole("switch");
    expect(switches.length).toBe(LAYERS.length);
  });

  it("clicking a toggle calls the store's toggleLayer", async () => {
    const user = userEvent.setup();
    render(<LayerControlPanel open={true} onClose={vi.fn()} />);
    // "洪水浸水" is layers.ts nameJa for the "flood" layer
    const floodSwitch = screen.getByText("洪水浸水").closest('[role="switch"]');
    if (!floodSwitch) throw new Error("flood switch not found");
    const before = useMapStore.getState().visibleLayers.has("flood");
    await user.click(floodSwitch);
    const after = useMapStore.getState().visibleLayers.has("flood");
    expect(after).toBe(!before);
  });

  it("close button calls onClose", async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    render(<LayerControlPanel open={true} onClose={onClose} />);
    await user.click(screen.getByRole("button", { name: /close panel/i }));
    expect(onClose).toHaveBeenCalled();
  });
});
