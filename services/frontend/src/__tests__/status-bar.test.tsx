import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { StatusBar } from "@/components/status-bar";

const defaultProps = {
  lat: 35.681,
  lng: 139.767,
  zoom: 12,
  isLoading: false,
  isDemoMode: false,
};

describe("StatusBar error indicators", () => {
  it("shows WASM error indicator when wasmError is true", () => {
    render(<StatusBar {...defaultProps} wasmError={true} />);
    expect(screen.getByText(/WASM/)).toBeInTheDocument();
  });

  it("hides WASM error indicator when wasmError is false", () => {
    render(<StatusBar {...defaultProps} wasmError={false} />);
    expect(screen.queryByText(/WASM/)).not.toBeInTheDocument();
  });

  it("shows area data error indicator when areaDataError is true", () => {
    render(<StatusBar {...defaultProps} areaDataError={true} />);
    expect(screen.getByText(/データ取得エラー/)).toBeInTheDocument();
  });

  it("hides area data error indicator when areaDataError is false", () => {
    render(<StatusBar {...defaultProps} areaDataError={false} />);
    expect(screen.queryByText(/データ取得エラー/)).not.toBeInTheDocument();
  });

  it("shows zoom warning when isZoomTooLow is true", () => {
    render(<StatusBar {...defaultProps} isZoomTooLow={true} />);
    expect(screen.getByText(/ズームイン/)).toBeInTheDocument();
  });

  it("shows both WASM and area data errors simultaneously", () => {
    render(
      <StatusBar {...defaultProps} wasmError={true} areaDataError={true} />,
    );
    expect(screen.getByText(/WASM/)).toBeInTheDocument();
    expect(screen.getByText(/データ取得エラー/)).toBeInTheDocument();
  });
});
