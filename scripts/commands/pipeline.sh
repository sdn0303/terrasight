#!/usr/bin/env bash
# =============================================================================
# pipeline.sh — Full data pipeline: download → convert → import → build static
# =============================================================================
# Usage:
#   ./scripts/commands/pipeline.sh                    # full pipeline
#   ./scripts/commands/pipeline.sh --skip-download    # skip download step
#   ./scripts/commands/pipeline.sh --skip-convert     # skip convert step
#   ./scripts/commands/pipeline.sh --import-only      # DB import + static build only
#
# Prerequisites:
#   - Docker db container running
#   - uv installed
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

SKIP_DOWNLOAD=false
SKIP_CONVERT=false
IMPORT_ONLY=false

for arg in "$@"; do
  case "$arg" in
    --skip-download) SKIP_DOWNLOAD=true ;;
    --skip-convert) SKIP_CONVERT=true ;;
    --import-only) IMPORT_ONLY=true ;;
  esac
done

echo "============================================================"
echo " Data Pipeline"
echo " Started: $(date '+%Y-%m-%d %H:%M:%S')"
echo "============================================================"

# Step 1: Download
if ! $IMPORT_ONLY && ! $SKIP_DOWNLOAD; then
  echo ""
  echo "=== Step 1: Download government data ==="
  bash "$SCRIPT_DIR/download-data.sh"
fi

# Step 2: Convert
if ! $IMPORT_ONLY && ! $SKIP_CONVERT; then
  echo ""
  echo "=== Step 2: Convert RAW → GeoJSON ==="
  cd "$ROOT"
  uv run scripts/tools/convert_geodata.py 2>&1 | grep -vE "UserWarning|passing .format|import warnings|return pyogrio|GeoSeries.notna|Given a GeoSeries|To further ignore|^$"
fi

# Step 3: DB reset + import
echo ""
echo "=== Step 3: Database reset + import ==="
bash "$SCRIPT_DIR/db-full-reset.sh"

# Step 4: Build static data (FlatGeobuf)
echo ""
echo "=== Step 4: Build static data (FlatGeobuf) ==="
cd "$ROOT"
uv run scripts/tools/build_static_data.py 2>&1 | grep -v "^DEBUG"

echo ""
echo "============================================================"
echo " Pipeline complete: $(date '+%Y-%m-%d %H:%M:%S')"
echo "============================================================"
