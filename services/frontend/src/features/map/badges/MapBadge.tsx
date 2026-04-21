import { Marker } from "react-map-gl/mapbox";

interface MapBadgeProps {
  longitude: number;
  latitude: number;
  label: string;
  value: string;
  color: string;
  minZoom?: number;
  maxZoom?: number;
  currentZoom: number;
}

/**
 * 楽待ち型マップバッジ。市区町村ポリゴンのcentroidに表示。
 * Ref: DESIGN.md Sec 5.8
 */
export function MapBadge({
  longitude,
  latitude,
  label,
  value,
  color,
  minZoom = 8,
  maxZoom = 12,
  currentZoom,
}: MapBadgeProps) {
  if (currentZoom < minZoom || currentZoom > maxZoom) return null;

  return (
    <Marker longitude={longitude} latitude={latitude} anchor="center">
      <div
        className="flex items-center gap-1 px-2.5 whitespace-nowrap select-none pointer-events-auto cursor-pointer"
        style={{
          height: 28,
          borderRadius: "var(--ts-badge-radius)",
          background: color,
          fontFamily: "var(--font-mono)",
          fontSize: 11,
          fontWeight: 500,
          color: "#111111",
        }}
      >
        <span>{label}</span>
        <span className="font-semibold">{value}</span>
      </div>
    </Marker>
  );
}
