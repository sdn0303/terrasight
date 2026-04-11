import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { StatusPill } from "@/components/ui/status-pill";

describe("StatusPill", () => {
  it("renders Hot with fire emoji prefix", () => {
    render(<StatusPill status="hot" />);
    expect(screen.getByText(/Hot/)).toBeInTheDocument();
    expect(screen.getByText(/🔥/)).toBeInTheDocument();
  });

  it("renders Warm", () => {
    render(<StatusPill status="warm" />);
    expect(screen.getByText(/Warm/)).toBeInTheDocument();
  });

  it("renders Neutral", () => {
    render(<StatusPill status="neutral" />);
    expect(screen.getByText(/Neutral/)).toBeInTheDocument();
  });

  it("renders Cold", () => {
    render(<StatusPill status="cold" />);
    expect(screen.getByText(/Cold/)).toBeInTheDocument();
  });
});
