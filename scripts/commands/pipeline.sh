#!/usr/bin/env bash
set -euo pipefail

PREF=${1:-13}
PRIORITY=${2:-P0}

echo "=== Pipeline v2: pref=$PREF priority=$PRIORITY ==="

# Step 1: Convert raw -> GeoJSON
echo "--- Step 1: Convert ---"
uv run scripts/tools/pipeline/convert.py --pref "$PREF" --priority "$PRIORITY"

# Step 2: Build FlatGeobuf + manifest
echo "--- Step 2: Build FGB ---"
uv run scripts/tools/pipeline/build_fgb.py --pref "$PREF"

# Step 3: Import to PostGIS
echo "--- Step 3: Import ---"
uv run scripts/tools/pipeline/import_db.py --pref "$PREF" --priority "$PRIORITY"

# Step 4: Validate
echo "--- Step 4: Validate ---"
uv run scripts/tools/pipeline/validate.py --pref "$PREF"

echo "=== Pipeline complete ==="
