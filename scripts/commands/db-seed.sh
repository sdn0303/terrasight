#!/usr/bin/env bash
# Seed the database with development data.
# Usage: ./scripts/commands/db-seed.sh
set -euo pipefail

MIGRATIONS_DIR="services/backend/migrations"

echo "=== Seeding database ==="
docker compose exec -T db psql -U app -d realestate < "$MIGRATIONS_DIR/20260322000001_seed_dev.sql"
docker compose exec -T db psql -U app -d realestate -c "ANALYZE;"
echo "=== Seed complete ==="

# Show row counts
docker compose exec db psql -U app -d realestate -c "
SELECT relname AS table_name, n_live_tup AS row_count
FROM pg_stat_user_tables
WHERE schemaname = 'public' AND relname NOT IN ('spatial_ref_sys')
ORDER BY relname;"
