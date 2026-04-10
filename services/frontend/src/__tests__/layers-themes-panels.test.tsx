import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { LayerToggleRow } from "@/components/layer/layer-toggle-row";

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
