import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { BottomSheet } from "@/components/layout/bottom-sheet";

describe("BottomSheet", () => {
  const baseProps = {
    open: true,
    onClose: vi.fn(),
    title: "Opportunities",
    heightPct: 40,
    onHeightChange: vi.fn(),
  };

  it("renders nothing when closed", () => {
    render(
      <BottomSheet {...baseProps} open={false}>
        <div>body</div>
      </BottomSheet>,
    );
    expect(screen.queryByRole("region")).toBeNull();
  });

  it("renders title when open", () => {
    render(
      <BottomSheet {...baseProps}>
        <div>body</div>
      </BottomSheet>,
    );
    expect(screen.getByText("Opportunities")).toBeInTheDocument();
  });

  it("renders children in the body", () => {
    render(
      <BottomSheet {...baseProps}>
        <div data-testid="body">body content</div>
      </BottomSheet>,
    );
    expect(screen.getByTestId("body")).toBeInTheDocument();
  });

  it("applies the heightPct to the root element style", () => {
    render(
      <BottomSheet {...baseProps} heightPct={60}>
        <div>body</div>
      </BottomSheet>,
    );
    const region = screen.getByRole("region") as HTMLElement;
    expect(region.style.height).toBe("60%");
  });

  it("renders drag handle with aria-valuenow", () => {
    render(
      <BottomSheet {...baseProps} heightPct={45}>
        <div>body</div>
      </BottomSheet>,
    );
    const handle = screen.getByRole("slider", { name: /resize bottom sheet/i });
    expect(handle.getAttribute("aria-valuenow")).toBe("45");
    expect(handle.getAttribute("aria-valuemin")).toBe("20");
    expect(handle.getAttribute("aria-valuemax")).toBe("80");
  });

  it("keyboard ArrowUp on drag handle increases height by 5", async () => {
    const onHeightChange = vi.fn();
    const user = userEvent.setup();
    render(
      <BottomSheet
        {...baseProps}
        heightPct={40}
        onHeightChange={onHeightChange}
      >
        <div>body</div>
      </BottomSheet>,
    );
    const handle = screen.getByRole("slider", { name: /resize bottom sheet/i });
    handle.focus();
    await user.keyboard("{ArrowUp}");
    expect(onHeightChange).toHaveBeenCalledWith(45);
  });

  it("keyboard ArrowDown on drag handle decreases height by 5", async () => {
    const onHeightChange = vi.fn();
    const user = userEvent.setup();
    render(
      <BottomSheet
        {...baseProps}
        heightPct={40}
        onHeightChange={onHeightChange}
      >
        <div>body</div>
      </BottomSheet>,
    );
    const handle = screen.getByRole("slider", { name: /resize bottom sheet/i });
    handle.focus();
    await user.keyboard("{ArrowDown}");
    expect(onHeightChange).toHaveBeenCalledWith(35);
  });

  it("clicking close calls onClose", async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    render(
      <BottomSheet {...baseProps} onClose={onClose}>
        <div>body</div>
      </BottomSheet>,
    );
    await user.click(
      screen.getByRole("button", { name: /close bottom sheet/i }),
    );
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("does not exceed 80 when ArrowUp at max", async () => {
    const onHeightChange = vi.fn();
    const user = userEvent.setup();
    render(
      <BottomSheet
        {...baseProps}
        heightPct={80}
        onHeightChange={onHeightChange}
      >
        <div>body</div>
      </BottomSheet>,
    );
    const handle = screen.getByRole("slider", { name: /resize bottom sheet/i });
    handle.focus();
    await user.keyboard("{ArrowUp}");
    expect(onHeightChange).toHaveBeenCalledWith(80);
  });

  it("does not go below 20 when ArrowDown at min", async () => {
    const onHeightChange = vi.fn();
    const user = userEvent.setup();
    render(
      <BottomSheet
        {...baseProps}
        heightPct={20}
        onHeightChange={onHeightChange}
      >
        <div>body</div>
      </BottomSheet>,
    );
    const handle = screen.getByRole("slider", { name: /resize bottom sheet/i });
    handle.focus();
    await user.keyboard("{ArrowDown}");
    expect(onHeightChange).toHaveBeenCalledWith(20);
  });
});
