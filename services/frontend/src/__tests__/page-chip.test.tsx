import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { PageChip } from "@/components/layout/page-chip";

describe("PageChip", () => {
  it("renders children", () => {
    render(
      <PageChip anchor="bottom-center">
        <span data-testid="child">chip content</span>
      </PageChip>,
    );
    expect(screen.getByTestId("child")).toBeInTheDocument();
  });

  it("positions at bottom-center", () => {
    const { container } = render(
      <PageChip anchor="bottom-center" aria-label="Time machine">
        content
      </PageChip>,
    );
    const el = container.firstChild as HTMLElement;
    expect(el.style.bottom).not.toBe("");
    expect(el.style.left).toBe("50%");
    expect(el.style.transform).toContain("translateX(-50%)");
  });

  it("positions at bottom-right", () => {
    const { container } = render(
      <PageChip anchor="bottom-right" aria-label="Status">
        content
      </PageChip>,
    );
    const el = container.firstChild as HTMLElement;
    expect(el.style.bottom).not.toBe("");
    expect(el.style.right).not.toBe("");
    expect(el.style.left).toBe("");
  });

  it("positions at top-right", () => {
    const { container } = render(
      <PageChip anchor="top-right" aria-label="Stats">
        content
      </PageChip>,
    );
    const el = container.firstChild as HTMLElement;
    expect(el.style.top).not.toBe("");
    expect(el.style.right).not.toBe("");
  });

  it("applies accessible label", () => {
    render(
      <PageChip anchor="bottom-center" aria-label="Time machine">
        content
      </PageChip>,
    );
    expect(screen.getByLabelText("Time machine")).toBeInTheDocument();
  });

  it("accepts custom offset overrides", () => {
    const { container } = render(
      <PageChip anchor="bottom-right" offset={{ bottom: 60, right: 30 }}>
        content
      </PageChip>,
    );
    const el = container.firstChild as HTMLElement;
    expect(el.style.bottom).toBe("60px");
    expect(el.style.right).toBe("30px");
  });
});
