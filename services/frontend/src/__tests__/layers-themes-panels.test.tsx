import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Shield } from "lucide-react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { LayerControlPanel } from "@/components/layer/layer-control-panel";
import { LayerToggleRow } from "@/components/layer/layer-toggle-row";
import { ThemeCard } from "@/components/theme/theme-card";
import { ThemesPanel } from "@/components/theme/themes-panel";
import { LAYERS } from "@/lib/layers";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

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

describe("ThemeCard", () => {
  const baseProps = {
    id: "safety" as const,
    label: "Safety",
    description: "Disaster risk visualization",
    layerCount: 7,
    icon: Shield,
    active: false,
    onToggle: vi.fn(),
  };

  it("renders label, description, and layer count", () => {
    render(<ThemeCard {...baseProps} />);
    expect(screen.getByText("Safety")).toBeInTheDocument();
    expect(screen.getByText("Disaster risk visualization")).toBeInTheDocument();
    expect(screen.getByText("7 layers")).toBeInTheDocument();
  });

  it("marks active card with aria-pressed=true", () => {
    render(<ThemeCard {...baseProps} active={true} />);
    expect(screen.getByRole("button").getAttribute("aria-pressed")).toBe(
      "true",
    );
  });

  it("clicking calls onToggle with id", async () => {
    const onToggle = vi.fn();
    const user = userEvent.setup();
    render(<ThemeCard {...baseProps} onToggle={onToggle} />);
    await user.click(screen.getByRole("button"));
    expect(onToggle).toHaveBeenCalledWith("safety");
  });
});

describe("ThemesPanel", () => {
  beforeEach(() => {
    useUIStore.setState({ activeThemes: new Set() });
    useMapStore.setState({ visibleLayers: new Set(defaultVisibleLayers) });
  });

  it("renders nothing when closed", () => {
    const { container } = render(
      <ThemesPanel open={false} onClose={vi.fn()} />,
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders the panel title and subtitle with selection count", () => {
    useUIStore.setState({ activeThemes: new Set(["safety", "price"]) });
    render(<ThemesPanel open={true} onClose={vi.fn()} />);
    expect(screen.getByText("Investment Themes")).toBeInTheDocument();
    expect(screen.getByText("2 selected")).toBeInTheDocument();
  });

  it("renders one card per theme (4 themes)", () => {
    render(<ThemesPanel open={true} onClose={vi.fn()} />);
    // 4 theme buttons with aria-pressed + 1 close button in LeftPanel
    const pressableButtons = screen
      .getAllByRole("button")
      .filter((el) => el.hasAttribute("aria-pressed"));
    expect(pressableButtons.length).toBe(4);
  });

  it("clicking a card toggles activeThemes and applies theme layers to map", async () => {
    const user = userEvent.setup();
    render(<ThemesPanel open={true} onClose={vi.fn()} />);
    // next-intl mock returns the key as-is, so the label for the safety
    // theme will be "theme.safety.name"
    const safetyBtn = screen.getByText("theme.safety.name").closest("button");
    if (!safetyBtn) throw new Error("safety card not found");
    await user.click(safetyBtn);
    expect(useUIStore.getState().activeThemes.has("safety")).toBe(true);
    // applyThemeLayers should have unioned in safety layers
    const visible = useMapStore.getState().visibleLayers;
    expect(visible.has("flood")).toBe(true); // flood is in safety theme
  });

  it("close button calls onClose", async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    render(<ThemesPanel open={true} onClose={onClose} />);
    await user.click(screen.getByRole("button", { name: /close panel/i }));
    expect(onClose).toHaveBeenCalled();
  });
});
