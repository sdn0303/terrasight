#!/usr/bin/env bash
set -euo pipefail

# seed-dev.sh — Populate the PostGIS database with minimal dev data.
#
# Usage:
#   ./scripts/seed-dev.sh                       # uses DATABASE_URL from env or .env
#   DATABASE_URL=postgres://... ./scripts/seed-dev.sh
#
# Prerequisites:
#   - PostgreSQL with PostGIS running
#   - Schema migration already applied (sqlx migrate run)
#   - psql available on PATH

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$(dirname "$SCRIPT_DIR")"

# Load .env if present
if [ -f "$BACKEND_DIR/.env" ]; then
  # shellcheck disable=SC1091
  source "$BACKEND_DIR/.env"
fi

DB_URL="${DATABASE_URL:?DATABASE_URL is required. Set it in .env or export it.}"

echo "==> Running schema migrations..."
cd "$BACKEND_DIR"
if command -v sqlx &>/dev/null; then
  sqlx migrate run --database-url "$DB_URL"
else
  echo "    sqlx-cli not found, attempting direct psql migration..."
  for migration in migrations/*.sql; do
    echo "    Applying $migration"
    psql "$DB_URL" -f "$migration" -v ON_ERROR_STOP=1
  done
fi

echo "==> Seeding dev data..."
psql "$DB_URL" -f "$BACKEND_DIR/migrations/20260322000001_seed_dev.sql" -v ON_ERROR_STOP=1

echo "==> Verifying row counts..."
psql "$DB_URL" -c "
  SELECT 'land_prices'        AS table_name, count(*) FROM land_prices
  UNION ALL
  SELECT 'zoning',                           count(*) FROM zoning
  UNION ALL
  SELECT 'flood_risk',                       count(*) FROM flood_risk
  UNION ALL
  SELECT 'steep_slope',                      count(*) FROM steep_slope
  UNION ALL
  SELECT 'schools',                          count(*) FROM schools
  UNION ALL
  SELECT 'medical_facilities',               count(*) FROM medical_facilities
  ORDER BY table_name;
"

echo "==> Dev seed complete!"
