import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ScoreChip } from "@/components/ui/score-chip";

// jsdom normalises inline-style colour literals to rgb(r, g, b). Matching
// on the normalised form keeps assertions stable across environments.

describe("ScoreChip", () => {
  it("renders the score value", () => {
    render(<ScoreChip value={82} />);
    expect(screen.getByText("82")).toBeInTheDocument();
  });

  it("applies success gradient for high scores (>= 80)", () => {
    const { container } = render(<ScoreChip value={85} />);
    const chip = container.firstChild as HTMLElement;
    expect(chip.style.background).toContain("rgb(16, 185, 129)");
  });

  it("applies brand gradient for 60-79 scores", () => {
    const { container } = render(<ScoreChip value={70} />);
    const chip = container.firstChild as HTMLElement;
    expect(chip.style.background).toContain("rgb(99, 102, 241)");
  });

  it("applies warn gradient for 40-59 scores", () => {
    const { container } = render(<ScoreChip value={50} />);
    const chip = container.firstChild as HTMLElement;
    expect(chip.style.background).toContain("rgb(245, 158, 11)");
  });

  it("applies danger gradient for scores below 40", () => {
    const { container } = render(<ScoreChip value={20} />);
    const chip = container.firstChild as HTMLElement;
    expect(chip.style.background).toContain("rgb(239, 68, 68)");
  });
});
