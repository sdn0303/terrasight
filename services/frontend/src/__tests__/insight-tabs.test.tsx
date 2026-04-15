import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AxisBreakdown } from "@/components/insight/axis-breakdown";
import { ScoreHeroCard } from "@/components/insight/score-hero-card";

describe("ScoreHeroCard", () => {
  it("renders the TLS score big and the Top X% pill", () => {
    render(
      <ScoreHeroCard
        tls={82}
        topPercentile={8}
        deltaVsArea={6}
        presetStats={{ balance: 76, residential: 79, disaster: 88 }}
      />,
    );
    expect(screen.getByText("82")).toBeInTheDocument();
    expect(screen.getByText(/Top 8%/)).toBeInTheDocument();
    expect(screen.getByText(/\+6/)).toBeInTheDocument();
  });

  it("renders the three preset mini stats", () => {
    render(
      <ScoreHeroCard
        tls={82}
        topPercentile={8}
        deltaVsArea={6}
        presetStats={{ balance: 76, residential: 79, disaster: 88 }}
      />,
    );
    expect(screen.getByText("76")).toBeInTheDocument();
    expect(screen.getByText("79")).toBeInTheDocument();
    expect(screen.getByText("88")).toBeInTheDocument();
  });

  it("shows negative delta with down arrow", () => {
    render(
      <ScoreHeroCard
        tls={42}
        topPercentile={62}
        deltaVsArea={-3}
        presetStats={{ balance: 40, residential: 42, disaster: 50 }}
      />,
    );
    expect(screen.getByText(/-3/)).toBeInTheDocument();
  });

  it("omits the + sign when delta is exactly zero", () => {
    render(
      <ScoreHeroCard
        tls={60}
        topPercentile={50}
        deltaVsArea={0}
        presetStats={{ balance: 60, residential: 60, disaster: 60 }}
      />,
    );
    // Neutral arrow + bare "0 vs area" without a leading "+".
    expect(screen.getByText(/— 0 vs area/)).toBeInTheDocument();
    expect(screen.queryByText(/\+0 vs area/)).toBeNull();
  });

  it("omits Top X% pill when topPercentile is null", () => {
    render(
      <ScoreHeroCard
        tls={50}
        topPercentile={null}
        deltaVsArea={0}
        presetStats={{ balance: 50, residential: 50, disaster: 50 }}
      />,
    );
    expect(screen.queryByText(/Top /)).toBeNull();
  });
});

describe("AxisBreakdown", () => {
  const axes = {
    disaster: 88,
    terrain: 75,
    livability: 84,
    future: 68,
    price: 79,
  };

  it("renders all 5 axis labels", () => {
    render(<AxisBreakdown axes={axes} />);
    expect(screen.getByText("災害")).toBeInTheDocument();
    expect(screen.getByText("地形")).toBeInTheDocument();
    expect(screen.getByText("生活")).toBeInTheDocument();
    expect(screen.getByText("将来")).toBeInTheDocument();
    expect(screen.getByText("価格")).toBeInTheDocument();
  });

  it("renders all 5 values", () => {
    render(<AxisBreakdown axes={axes} />);
    expect(screen.getByText("88")).toBeInTheDocument();
    expect(screen.getByText("75")).toBeInTheDocument();
    expect(screen.getByText("84")).toBeInTheDocument();
    expect(screen.getByText("68")).toBeInTheDocument();
    expect(screen.getByText("79")).toBeInTheDocument();
  });
});
