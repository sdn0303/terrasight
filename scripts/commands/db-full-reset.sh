#!/usr/bin/env bash
# =============================================================================
# db-full-reset.sh — One-command DB reset: migrate + import all data + ANALYZE
# =============================================================================
# Usage: ./scripts/commands/db-full-reset.sh
#
# This is the "do everything" script. It:
#   1. Drops all tables and applies the canonical schema migration
#   2. Optionally runs pipeline import for Tokyo (pref 13)
#   3. Runs ANALYZE for query planner stats
#   4. Shows final row counts
#
# Prerequisites:
#   - Docker db container running: docker compose up -d db
#   - GeoJSON files in data/geojson/ (run pipeline convert first)
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

# --- Step 1: Drop all tables and re-create schema ---
echo ""
echo "--- Step 1: Drop existing tables + apply schema ---"
docker compose exec -T db psql -U app -d realestate -c "
DROP SCHEMA public CASCADE;
CREATE SCHEMA public;
GRANT ALL ON SCHEMA public TO app;
" 2>&1 | tail -1

docker compose exec -T db psql -U app -d realestate < "$MIGRATIONS/00000000000001_schema.sql" 2>&1 | grep -c "CREATE" | xargs -I{} echo "  {} CREATE statements executed"

# --- Step 2: Seed (if exists) ---
if [ -f "$MIGRATIONS/00000000000002_seed.sql" ]; then
    echo ""
    echo "--- Step 2: Apply seed data ---"
    docker compose exec -T db psql -U app -d realestate < "$MIGRATIONS/00000000000002_seed.sql" 2>&1 | tail -1
fi

# --- Step 3: Import via pipeline (if GeoJSON exists) ---
echo ""
echo "--- Step 3: Import data via pipeline ---"
cd "$ROOT"
if [ -d "data/geojson/13" ] && [ "$(ls -A data/geojson/13/ 2>/dev/null)" ]; then
    uv run scripts/tools/pipeline/import_db.py --pref 13 2>&1 | grep -E "(Imported|Deleted|ERROR|complete)" || true
else
    echo "  No GeoJSON data found. Run pipeline convert first:"
    echo "    uv run scripts/tools/pipeline/convert.py --pref 13 --priority P0"
fi

# --- Step 4: ANALYZE ---
echo ""
echo "--- Step 4: ANALYZE ---"
docker compose exec -T db psql -U app -d realestate -c "ANALYZE;" > /dev/null 2>&1
echo "  ANALYZE complete"

# --- Step 5: Show results ---
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
