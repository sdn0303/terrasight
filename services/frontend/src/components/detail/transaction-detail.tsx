"use client";

interface Props {
  lat: number;
  lng: number;
  featureProperties?: Record<string, unknown>;
}

function formatPropertyValue(value: unknown): string {
  if (value === null || value === undefined) return "—";
  if (typeof value === "number") return value.toLocaleString();
  if (typeof value === "string") return value;
  if (typeof value === "boolean") return value ? "あり" : "なし";
  return String(value);
}

export function TransactionDetail({ featureProperties }: Props) {
  const hasProperties =
    featureProperties !== undefined &&
    Object.keys(featureProperties).length > 0;

  return (
    <div className="space-y-4">
      {hasProperties ? (
        <div>
          <p
            className="mb-2 text-[10px] font-semibold uppercase tracking-wider"
            style={{ color: "var(--panel-text-secondary)" }}
          >
            取引データ
          </p>
          <table className="w-full">
            <tbody>
              {Object.entries(featureProperties).map(([key, value]) => (
                <tr key={key}>
                  <td
                    className="py-1.5 pr-3 text-xs w-1/2"
                    style={{ color: "var(--panel-text-secondary)" }}
                  >
                    {key}
                  </td>
                  <td
                    className="py-1.5 text-xs font-medium"
                    style={{ color: "var(--panel-text-primary)" }}
                  >
                    {formatPropertyValue(value)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center py-8 space-y-2">
          <div
            className="rounded-full flex items-center justify-center w-12 h-12"
            style={{ background: "var(--panel-hover-bg)" }}
          >
            <span
              className="text-xl"
              role="img"
              aria-label="construction"
            >
              🔨
            </span>
          </div>
          <p
            className="text-sm font-semibold"
            style={{ color: "var(--panel-text-primary)" }}
          >
            準備中
          </p>
          <p
            className="text-xs text-center"
            style={{ color: "var(--panel-text-secondary)" }}
          >
            取引事例API はフェーズ3cで実装予定です
          </p>
        </div>
      )}
    </div>
  );
}
