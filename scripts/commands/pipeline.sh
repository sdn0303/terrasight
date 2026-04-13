#!/usr/bin/env bash
set -euo pipefail

PREF=${1:-13}
PRIORITY=${2:-P0}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
MIGRATIONS="$ROOT/services/backend/migrations"

echo "=== Pipeline v2: pref=$PREF priority=$PRIORITY ==="

# Step 0: Ensure DB schema exists (idempotent — skips if tables already exist)
echo "--- Step 0: Ensure DB schema ---"
if docker compose exec -T db psql -U app -d realestate -c "SELECT 1 FROM admin_boundaries LIMIT 0" > /dev/null 2>&1; then
    echo "  Schema already applied"
else
    echo "  Applying schema migration..."
    docker compose exec -T db psql -U app -d realestate -c "
        DROP SCHEMA IF EXISTS public CASCADE;
        CREATE SCHEMA public;
        GRANT ALL ON SCHEMA public TO app;
    " > /dev/null 2>&1
    docker compose exec -T db psql -U app -d realestate < "$MIGRATIONS/00000000000001_schema.sql" > /dev/null 2>&1
    echo "  Schema applied"
fi

# Step 0b: Ensure REINFOLIB schema (additive — safe to re-run)
if docker compose exec -T db psql -U app -d realestate -c "SELECT 1 FROM transaction_prices LIMIT 0" > /dev/null 2>&1; then
    echo "  REINFOLIB schema already applied"
else
    echo "  Applying REINFOLIB migration..."
    docker compose exec -T db psql -U app -d realestate < "$MIGRATIONS/00000000000002_reinfolib.sql" > /dev/null 2>&1
    echo "  REINFOLIB schema applied"
fi

# Step 0c: Ensure e-Stat schema (additive — safe to re-run)
if docker compose exec -T db psql -U app -d realestate -c "SELECT 1 FROM population_municipality LIMIT 0" > /dev/null 2>&1; then
    echo "  e-Stat schema already applied"
else
    echo "  Applying e-Stat migration..."
    docker compose exec -T db psql -U app -d realestate < "$MIGRATIONS/00000000000003_estat.sql" > /dev/null 2>&1
    echo "  e-Stat schema applied"
fi

# Step 1: Convert raw -> GeoJSON
echo "--- Step 1: Convert ---"
uv run scripts/tools/pipeline/convert.py --pref "$PREF" --priority "$PRIORITY"

# Step 2: Build FlatGeobuf + manifest
echo "--- Step 2: Build FGB ---"
uv run scripts/tools/pipeline/build_fgb.py --pref "$PREF"

# Step 3: Import to PostGIS
echo "--- Step 3: Import ---"
export DATABASE_URL="postgresql://app:${DB_PASSWORD:-devpass}@localhost:5432/realestate"
uv run scripts/tools/pipeline/import_db.py --pref "$PREF" --priority "$PRIORITY"

# Step 3b: Import REINFOLIB data
echo "--- Step 3b: Import REINFOLIB ---"
uv run scripts/tools/pipeline/import_db.py --pref "$PREF" --reinfolib

# Step 3c: e-Stat data import
echo "--- Step 3c: Import e-Stat ---"
uv run scripts/tools/pipeline/import_db.py --pref "$PREF" --estat

# Step 4: Validate
echo "--- Step 4: Validate ---"
uv run scripts/tools/pipeline/validate.py --pref "$PREF"

echo "=== Pipeline complete ==="
