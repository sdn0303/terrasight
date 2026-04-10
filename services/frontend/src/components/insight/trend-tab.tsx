"use client";

import { TrendChart } from "@/components/analyze/trend-chart";

interface TrendTabProps {
  lat: number;
  lng: number;
}

export function TrendTab({ lat, lng }: TrendTabProps) {
  return (
    <div className="space-y-4">
      <div>
        <div
          className="mb-2 text-[8px] font-extrabold uppercase"
          style={{ color: "var(--neutral-400)", letterSpacing: "0.7px" }}
        >
          Price Trend
        </div>
        <TrendChart lat={lat} lng={lng} />
      </div>
    </div>
  );
}
