#!/usr/bin/env bash
# Apply all SQL migrations to the database in order.
# Usage: ./scripts/commands/db-migrate.sh
set -euo pipefail

MIGRATIONS_DIR="services/backend/migrations"
DB_CONTAINER="sample-app-db-1"

echo "=== Applying migrations ==="

# Apply schema redesign (destructive — drops and recreates all tables)
echo "--- 20260326000001_schema_redesign.sql ---"
docker compose exec -T db psql -U app -d realestate < "$MIGRATIONS_DIR/20260326000001_schema_redesign.sql"

# Apply admin boundaries table
echo "--- 20260326000002_admin_boundaries.sql ---"
docker compose exec -T db psql -U app -d realestate < "$MIGRATIONS_DIR/20260326000002_admin_boundaries.sql"

echo "=== Migrations complete ==="
