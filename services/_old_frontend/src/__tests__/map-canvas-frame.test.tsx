import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { MapCanvasFrame } from "@/components/layout/map-canvas-frame";

describe("MapCanvasFrame", () => {
  it("renders children", () => {
    render(
      <MapCanvasFrame>
        <div data-testid="child">Hello</div>
      </MapCanvasFrame>,
    );
    expect(screen.getByTestId("child")).toBeInTheDocument();
  });

  it("renders as a full-viewport positioned container", () => {
    const { container } = render(
      <MapCanvasFrame>
        <div>Hello</div>
      </MapCanvasFrame>,
    );
    const frame = container.firstChild as HTMLElement;
    expect(frame).toBeInstanceOf(HTMLElement);
    expect(frame.className).toContain("relative");
    expect(frame.className).toMatch(/h-screen|h-\[100vh\]/);
    expect(frame.className).toMatch(/w-screen|w-\[100vw\]/);
  });

  it("sets an aria-label for screen readers", () => {
    const { container } = render(
      <MapCanvasFrame aria-label="Terrasight map canvas">
        <div>Hello</div>
      </MapCanvasFrame>,
    );
    const frame = container.firstChild as HTMLElement;
    expect(frame.getAttribute("aria-label")).toBe("Terrasight map canvas");
  });
});
