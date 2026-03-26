#!/usr/bin/env bash
# scripts/download-l01-landprice.sh
# Downloads L01 land price data from NLNI
# Source: https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-L01-2026.html
# Usage: ./download-l01-landprice.sh [year]
set -euo pipefail

YEAR="${1:-2026}"
DATA_DIR="data/raw/l01/$YEAR"
TOTAL=0
FAILED=0

mkdir -p "$DATA_DIR"

echo "[L01] Downloading land price data ($YEAR)..."

for code in $(seq -w 1 47); do
  # Zero-pad to 2 digits
  pref_code=$(printf "%02d" "$code")
  url="https://nlftp.mlit.go.jp/ksj/gml/data/L01/L01-${YEAR}/L01-${YEAR}_${pref_code}_GML.zip"
  outfile="$DATA_DIR/L01-${YEAR}_${pref_code}.zip"

  if [ -f "$outfile" ]; then
    echo "  [${pref_code}] Already exists, skipping"
    TOTAL=$((TOTAL + 1))
    continue
  fi

  if curl -sL --fail -o "$outfile" "$url" 2>/dev/null; then
    echo "  [${pref_code}] Downloaded: $(du -h "$outfile" | cut -f1)"
    TOTAL=$((TOTAL + 1))
  else
    echo "  [${pref_code}] Not available (HTTP error)"
    rm -f "$outfile"
    FAILED=$((FAILED + 1))
  fi
done

echo ""
echo "[L01] Complete: $TOTAL downloaded, $FAILED not available"
echo "[L01] Files in $DATA_DIR/"
