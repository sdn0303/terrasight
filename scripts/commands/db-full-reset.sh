#!/usr/bin/env bash
# =============================================================================
# db-full-reset.sh — One-command DB reset: migrate + import all data + ANALYZE
# =============================================================================
# Usage: ./scripts/commands/db-full-reset.sh
#
# This is the "do everything" script. It:
#   1. Applies schema migration (destructive — drops all tables)
#   2. Applies admin_boundaries migration
#   3. Imports all GeoJSON datasets (9 datasets, ~700K rows)
#   4. Imports L01 land prices (5 years, ~20K rows)
#   5. Runs ANALYZE for query planner stats
#   6. Shows final row counts
#
# Prerequisites:
#   - Docker db container running: docker compose up -d db
#   - GeoJSON files in data/geojson/ (run convert_geodata.py first)
#   - uv installed for Python script execution
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
MIGRATIONS="$ROOT/services/backend/migrations"
export DATABASE_URL="postgresql://app:${DB_PASSWORD:-devpass}@localhost:5432/realestate"

echo "============================================================"
echo " Full Database Reset"
echo " Started: $(date '+%Y-%m-%d %H:%M:%S')"
echo "============================================================"

# --- Step 1: Schema migration ---
echo ""
echo "--- Step 1: Apply schema migration (destructive) ---"
docker compose exec -T db psql -U app -d realestate < "$MIGRATIONS/20260326000001_schema_redesign.sql" 2>&1 | grep -c "CREATE" | xargs -I{} echo "  {} CREATE statements executed"

echo ""
echo "--- Step 2: Apply admin_boundaries migration ---"
docker compose exec -T db psql -U app -d realestate < "$MIGRATIONS/20260326000002_admin_boundaries.sql" 2>&1 | grep -cE "CREATE|already exists" | xargs -I{} echo "  {} statements executed"

# --- Step 3: Import GeoJSON ---
echo ""
echo "--- Step 3: Import GeoJSON datasets ---"
cd "$ROOT"
uv run scripts/tools/import_geojson.py 2>&1 | grep -E "(Inserted|ERROR|Done)" || true

# --- Step 4: Import L01 ---
echo ""
echo "--- Step 4: Import L01 land prices ---"
uv run scripts/tools/import_l01.py 2>&1 | grep -E "(Inserted|ERROR|Done)" || true

# --- Step 5: ANALYZE ---
echo ""
echo "--- Step 5: ANALYZE ---"
docker compose exec -T db psql -U app -d realestate -c "ANALYZE;" > /dev/null 2>&1
echo "  ANALYZE complete"

# --- Step 6: Show results ---
echo ""
echo "--- Final row counts ---"
docker compose exec db psql -U app -d realestate -c "
SELECT relname AS table_name, n_live_tup AS rows
FROM pg_stat_user_tables
WHERE schemaname = 'public' AND relname NOT IN ('spatial_ref_sys')
ORDER BY n_live_tup DESC;"

echo ""
echo "============================================================"
echo " Database reset complete: $(date '+%Y-%m-%d %H:%M:%S')"
echo "============================================================"
