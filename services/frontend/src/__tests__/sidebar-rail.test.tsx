import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import { SidebarRail } from "@/components/layout/sidebar-rail";
import { useUIStore } from "@/stores/ui-store";

describe("SidebarRail", () => {
  beforeEach(() => {
    useUIStore.setState({
      leftPanel: null,
      bottomSheet: null,
      insight: null,
      settingsOpen: false,
    });
  });

  it("renders the brand mark", () => {
    render(<SidebarRail />);
    expect(screen.getByLabelText("Terrasight")).toBeInTheDocument();
  });

  it("renders all tool buttons with labels", () => {
    render(<SidebarRail />);
    expect(screen.getByRole("button", { name: /map/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /finder/i })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /opportunities/i }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /layers/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /themes/i })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /settings/i }),
    ).toBeInTheDocument();
  });

  it("clicking Finder tool sets leftPanel to finder", async () => {
    const user = userEvent.setup();
    render(<SidebarRail />);
    await user.click(screen.getByRole("button", { name: /finder/i }));
    expect(useUIStore.getState().leftPanel).toBe("finder");
  });

  it("clicking Finder tool twice closes the panel", async () => {
    const user = userEvent.setup();
    render(<SidebarRail />);
    const finder = screen.getByRole("button", { name: /finder/i });
    await user.click(finder);
    expect(useUIStore.getState().leftPanel).toBe("finder");
    await user.click(finder);
    expect(useUIStore.getState().leftPanel).toBeNull();
  });

  it("clicking Layers when Finder is open switches to Layers (exclusive)", async () => {
    const user = userEvent.setup();
    render(<SidebarRail />);
    await user.click(screen.getByRole("button", { name: /finder/i }));
    await user.click(screen.getByRole("button", { name: /layers/i }));
    expect(useUIStore.getState().leftPanel).toBe("layers");
  });

  it("clicking Opportunities toggles the bottom sheet", async () => {
    const user = userEvent.setup();
    render(<SidebarRail />);
    const opps = screen.getByRole("button", { name: /opportunities/i });
    await user.click(opps);
    expect(useUIStore.getState().bottomSheet).toBe("opportunities");
    await user.click(opps);
    expect(useUIStore.getState().bottomSheet).toBeNull();
  });

  it("clicking Map tool clears all overlays", async () => {
    useUIStore.setState({
      leftPanel: "finder",
      bottomSheet: "opportunities",
      insight: { kind: "point", lat: 35, lng: 139 },
    });
    const user = userEvent.setup();
    render(<SidebarRail />);
    await user.click(screen.getByRole("button", { name: /map/i }));
    const s = useUIStore.getState();
    expect(s.leftPanel).toBeNull();
    expect(s.bottomSheet).toBeNull();
    expect(s.insight).toBeNull();
  });

  it("clicking Settings opens the settings modal", async () => {
    const user = userEvent.setup();
    render(<SidebarRail />);
    await user.click(screen.getByRole("button", { name: /settings/i }));
    expect(useUIStore.getState().settingsOpen).toBe(true);
  });

  it("marks the active tool with aria-pressed", () => {
    useUIStore.setState({ leftPanel: "finder" });
    render(<SidebarRail />);
    const finder = screen.getByRole("button", { name: /finder/i });
    expect(finder.getAttribute("aria-pressed")).toBe("true");
  });

  it("Map tool is not active when the insight drawer is open", () => {
    useUIStore.setState({
      leftPanel: null,
      bottomSheet: null,
      insight: { kind: "point", lat: 35, lng: 139 },
    });
    render(<SidebarRail />);
    const map = screen.getByRole("button", { name: /map/i });
    expect(map.getAttribute("aria-pressed")).toBe("false");
  });

  it("Map tool is active only when every overlay is closed", () => {
    useUIStore.setState({
      leftPanel: null,
      bottomSheet: null,
      insight: null,
    });
    render(<SidebarRail />);
    const map = screen.getByRole("button", { name: /map/i });
    expect(map.getAttribute("aria-pressed")).toBe("true");
  });
});
