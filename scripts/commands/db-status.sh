#!/usr/bin/env bash
# =============================================================================
# db-status.sh — Show database health, row counts, index usage, slow queries
# =============================================================================
# Usage: ./scripts/commands/db-status.sh
# =============================================================================
set -euo pipefail

PSQL="docker compose exec db psql -U app -d realestate"

echo "=== Database Status ==="
echo ""

echo "--- Connection ---"
$PSQL -c "SELECT version();" -t | head -1
echo ""

echo "--- Tables & Row Counts ---"
$PSQL -c "
SELECT relname AS table_name,
       n_live_tup AS rows,
       pg_size_pretty(pg_total_relation_size(relid)) AS total_size
FROM pg_stat_user_tables
WHERE schemaname = 'public' AND relname NOT IN ('spatial_ref_sys')
ORDER BY pg_total_relation_size(relid) DESC;"
echo ""

echo "--- Index Usage (top 10) ---"
$PSQL -c "
SELECT indexrelname AS index_name,
       idx_scan AS scans,
       idx_tup_read AS rows_read
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
ORDER BY idx_scan DESC
LIMIT 10;"
echo ""

echo "--- Slow Queries (pg_stat_statements, top 5) ---"
$PSQL -c "
SELECT LEFT(query, 80) AS query,
       calls,
       round(mean_exec_time::numeric, 2) AS avg_ms,
       round(total_exec_time::numeric, 2) AS total_ms
FROM pg_stat_statements
WHERE query NOT LIKE '%pg_stat%'
ORDER BY mean_exec_time DESC
LIMIT 5;" 2>/dev/null || echo "  pg_stat_statements not available"
echo ""

echo "--- Config ---"
$PSQL -t -c "SELECT 'work_mem: ' || current_setting('work_mem');"
$PSQL -t -c "SELECT 'random_page_cost: ' || current_setting('random_page_cost');"
$PSQL -t -c "SELECT 'shared_buffers: ' || current_setting('shared_buffers');"
$PSQL -t -c "SELECT 'statement_timeout: ' || current_setting('statement_timeout');"
echo ""
echo "=== Done ==="
