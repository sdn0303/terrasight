import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { FinderPanel } from "@/components/finder/finder-panel";
import { useFilterStore } from "@/stores/filter-store";

describe("FinderPanel", () => {
  beforeEach(() => {
    useFilterStore.getState().reset();
  });

  it("renders section headers", () => {
    render(
      <FinderPanel
        open={true}
        onClose={vi.fn()}
        onSearch={vi.fn()}
        matchCount={1247}
      />,
    );
    expect(screen.getByText(/● AREA/)).toBeInTheDocument();
    expect(screen.getByText(/● CRITERIA/)).toBeInTheDocument();
    expect(screen.getByText(/● ZONING & ACCESS/)).toBeInTheDocument();
    expect(screen.getByText(/● WEIGHT PRESET/)).toBeInTheDocument();
  });

  it("shows the match count in the header and CTA", () => {
    render(
      <FinderPanel
        open={true}
        onClose={vi.fn()}
        onSearch={vi.fn()}
        matchCount={1247}
      />,
    );
    expect(screen.getAllByText(/1,247/).length).toBeGreaterThan(0);
  });

  it("Search CTA calls onSearch", async () => {
    const onSearch = vi.fn();
    const user = userEvent.setup();
    render(
      <FinderPanel
        open={true}
        onClose={vi.fn()}
        onSearch={onSearch}
        matchCount={1247}
      />,
    );
    await user.click(screen.getByRole("button", { name: /物件を検索/ }));
    expect(onSearch).toHaveBeenCalled();
  });

  it("returns null when closed", () => {
    const { container } = render(
      <FinderPanel
        open={false}
        onClose={vi.fn()}
        onSearch={vi.fn()}
        matchCount={0}
      />,
    );
    expect(container).toBeEmptyDOMElement();
  });
});
