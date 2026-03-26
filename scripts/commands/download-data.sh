#!/usr/bin/env bash
# Download government data from NLNI and other sources.
# Usage: ./scripts/commands/download-data.sh [--skip-existing]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA_DIR="$ROOT/data/raw"

mkdir -p "$DATA_DIR"

echo "=== Downloading government data ==="

# N03 Administrative Boundaries (2025)
N03_ZIP="$DATA_DIR/N03-2025.zip"
if [ ! -f "$N03_ZIP" ]; then
  echo "--- N03: Administrative Boundaries ---"
  curl -L --progress-bar -o "$N03_ZIP" \
    "https://nlftp.mlit.go.jp/ksj/gml/data/N03/N03-2025/N03-20250101_GML.zip"
  echo "Downloaded: $(du -h "$N03_ZIP" | cut -f1)"
else
  echo "--- N03: Already exists, skipping ---"
fi

# L01 Land Prices (2026, per-prefecture)
YEAR="${1:-2026}"
echo "--- L01: Land Prices ($YEAR) ---"
for code in $(seq -w 1 47); do
  pref_code=$(printf "%02d" "$code")
  url="https://nlftp.mlit.go.jp/ksj/gml/data/L01/L01-${YEAR}/L01-${YEAR}_${pref_code}_GML.zip"
  outfile="$DATA_DIR/L01-${YEAR}_${pref_code}.zip"
  if [ -f "$outfile" ]; then
    continue
  fi
  if curl -sL --fail -o "$outfile" "$url" 2>/dev/null; then
    echo "  [${pref_code}] Downloaded"
  else
    rm -f "$outfile"
  fi
done

echo "=== Download complete ==="
echo "Files in $DATA_DIR: $(ls "$DATA_DIR"/*.zip 2>/dev/null | wc -l) zip files"
