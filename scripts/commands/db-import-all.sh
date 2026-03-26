#!/usr/bin/env bash
# =============================================================================
# db-import-all.sh — Import all data into PostGIS (GeoJSON + L01)
# =============================================================================
# Usage: ./scripts/commands/db-import-all.sh [--dry-run]
#
# Imports:
#   1. 9 GeoJSON datasets (zoning, flood, schools, medical, etc.)
#   2. L01 land prices (5 years)
#   3. Runs ANALYZE
#
# Does NOT run migrations. Use db-full-reset.sh for migrate + import.
# =============================================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export DATABASE_URL="postgresql://app:${DB_PASSWORD:-devpass}@localhost:5432/realestate"

DRY_RUN=""
for arg in "$@"; do
  if [ "$arg" = "--dry-run" ]; then
    DRY_RUN="--dry-run"
  fi
done

echo "=== Importing all data into PostGIS ==="
echo ""

cd "$ROOT"

echo "--- GeoJSON datasets (9) ---"
uv run scripts/tools/import_geojson.py $DRY_RUN 2>&1 | grep -v "^DEBUG"

echo ""
echo "--- L01 land prices ---"
uv run scripts/tools/import_l01.py $DRY_RUN 2>&1 | grep -v "^DEBUG"

if [ -z "$DRY_RUN" ]; then
  echo ""
  echo "--- ANALYZE ---"
  docker compose exec -T db psql -U app -d realestate -c "ANALYZE;" > /dev/null 2>&1
  echo "  ANALYZE complete"

  echo ""
  echo "--- Row counts ---"
  docker compose exec db psql -U app -d realestate -c "
  SELECT relname AS table_name, n_live_tup AS rows
  FROM pg_stat_user_tables
  WHERE schemaname = 'public' AND relname NOT IN ('spatial_ref_sys')
  ORDER BY n_live_tup DESC;"
fi

echo ""
echo "=== Import complete ==="
