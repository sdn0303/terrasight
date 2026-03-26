#!/usr/bin/env bash
# Full data pipeline: download -> convert -> import -> analyze
# Usage: ./scripts/commands/pipeline.sh [--skip-download] [--skip-convert]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== Data Pipeline ==="
echo "Started: $(date '+%Y-%m-%d %H:%M:%S')"

# Step 1: Download
if [[ "${1:-}" != "--skip-download" ]]; then
  echo ""
  echo "--- Step 1: Download ---"
  bash "$SCRIPT_DIR/download-data.sh"
fi

# Step 2: Convert raw data to GeoJSON
if [[ "${1:-}" != "--skip-convert" ]]; then
  echo ""
  echo "--- Step 2: Convert to GeoJSON ---"
  cd "$ROOT"
  uv run scripts/tools/convert_geodata.py
fi

# Step 3: Reset DB and import
echo ""
echo "--- Step 3: Database ---"
bash "$SCRIPT_DIR/db-reset.sh"

# Step 4: Import full data
echo ""
echo "--- Step 4: Import GeoJSON ---"
bash "$SCRIPT_DIR/db-import.sh"

# Step 5: Build static data (FlatGeobuf)
echo ""
echo "--- Step 5: Build static data ---"
cd "$ROOT"
uv run scripts/tools/build_static_data.py

echo ""
echo "=== Pipeline complete ==="
echo "Finished: $(date '+%Y-%m-%d %H:%M:%S')"
