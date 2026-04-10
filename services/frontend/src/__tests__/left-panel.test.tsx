import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { LeftPanel } from "@/components/layout/left-panel";

describe("LeftPanel", () => {
  const baseProps = {
    open: true,
    onClose: vi.fn(),
    title: "Investment Finder",
    subtitle: "1,247 properties match",
    badge: "FINDER",
  };

  it("renders nothing when closed", () => {
    const { container } = render(
      <LeftPanel {...baseProps} open={false}>
        <div>body</div>
      </LeftPanel>,
    );
    expect(container.querySelector('[role="region"]')).toBeNull();
  });

  it("renders title and subtitle when open", () => {
    render(
      <LeftPanel {...baseProps}>
        <div>body</div>
      </LeftPanel>,
    );
    expect(screen.getByText("Investment Finder")).toBeInTheDocument();
    expect(screen.getByText("1,247 properties match")).toBeInTheDocument();
  });

  it("renders the badge label uppercase", () => {
    render(
      <LeftPanel {...baseProps}>
        <div>body</div>
      </LeftPanel>,
    );
    expect(screen.getByText(/FINDER/)).toBeInTheDocument();
  });

  it("renders children inside the body", () => {
    render(
      <LeftPanel {...baseProps}>
        <div data-testid="body-child">Hello body</div>
      </LeftPanel>,
    );
    expect(screen.getByTestId("body-child")).toBeInTheDocument();
  });

  it("renders optional footer", () => {
    render(
      <LeftPanel {...baseProps} footer={<div data-testid="footer">footer</div>}>
        <div>body</div>
      </LeftPanel>,
    );
    expect(screen.getByTestId("footer")).toBeInTheDocument();
  });

  it("clicking the close button calls onClose", async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    render(
      <LeftPanel {...baseProps} onClose={onClose}>
        <div>body</div>
      </LeftPanel>,
    );
    await user.click(screen.getByRole("button", { name: /close panel/i }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("sets role region with aria-label", () => {
    render(
      <LeftPanel {...baseProps}>
        <div>body</div>
      </LeftPanel>,
    );
    const region = screen.getByRole("region", { name: /investment finder/i });
    expect(region).toBeInTheDocument();
  });
});
