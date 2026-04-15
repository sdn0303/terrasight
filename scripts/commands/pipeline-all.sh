#!/usr/bin/env bash
# =============================================================================
# pipeline-all.sh — Run data pipeline for all 47 prefectures
# =============================================================================
# Usage: ./scripts/commands/pipeline-all.sh [--priority P0|P1|P2|all] [--skip-reset]
#
# Examples:
#   ./scripts/commands/pipeline-all.sh                    # P0 only (default)
#   ./scripts/commands/pipeline-all.sh --priority all     # P0 + P1 + P2
#   ./scripts/commands/pipeline-all.sh --priority P1      # P1 only
#   ./scripts/commands/pipeline-all.sh --skip-reset       # Skip DB reset
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

PRIORITY="P0"
SKIP_RESET=false

for arg in "$@"; do
  case "$arg" in
    --priority) shift; PRIORITY="${1:-P0}"; shift || true ;;
    --skip-reset) SKIP_RESET=true ;;
    P0|P1|P2|all) PRIORITY="$arg" ;;
    *) echo "Unknown argument: $arg"; exit 1 ;;
  esac
done

cd "$ROOT"

echo "=== Pipeline All: priority=$PRIORITY skip_reset=$SKIP_RESET ==="
echo ""

# Step 1: Ensure DB is running
echo "--- Step 1: Ensure DB is running ---"
if ! docker compose exec -T db pg_isready -U app -d realestate > /dev/null 2>&1; then
  echo "  Starting DB..."
  docker compose up -d db
  echo "  Waiting for DB to be ready..."
  for i in $(seq 1 30); do
    if docker compose exec -T db pg_isready -U app -d realestate > /dev/null 2>&1; then
      echo "  DB ready"
      break
    fi
    sleep 2
  done
fi

# Step 2: DB reset (unless skipped)
if [ "$SKIP_RESET" = false ]; then
  echo "--- Step 2: DB full reset ---"
  "$SCRIPT_DIR/db-full-reset.sh"
else
  echo "--- Step 2: Skipped (--skip-reset) ---"
fi

# Step 3: Run pipeline for all prefectures
PRIORITIES=()
if [ "$PRIORITY" = "all" ]; then
  PRIORITIES=(P0 P1 P2)
else
  PRIORITIES=("$PRIORITY")
fi

FAILED=()
TOTAL=0
SUCCESS=0

for prio in "${PRIORITIES[@]}"; do
  echo ""
  echo "========================================="
  echo "  Priority: $prio"
  echo "========================================="
  for pref in $(seq -w 1 47); do
    TOTAL=$((TOTAL + 1))
    echo ""
    echo "--- Prefecture $pref ($prio) ---"
    if "$SCRIPT_DIR/pipeline.sh" "$pref" "$prio"; then
      SUCCESS=$((SUCCESS + 1))
    else
      echo "  WARN: Prefecture $pref $prio failed"
      FAILED+=("$pref:$prio")
    fi
  done
done

# Step 4: Refresh materialized views + ANALYZE
echo ""
echo "--- Step 4: Refresh materialized views + ANALYZE ---"
docker compose exec -T db psql -U app -d realestate -c "
  REFRESH MATERIALIZED VIEW CONCURRENTLY mv_transaction_summary;
  REFRESH MATERIALIZED VIEW CONCURRENTLY mv_appraisal_summary;
  ANALYZE;
" > /dev/null 2>&1
echo "  MV refresh + ANALYZE complete"

# Step 5: Row counts
echo ""
echo "--- Row counts ---"
docker compose exec -T db psql -U app -d realestate -c "
  SELECT relname AS table_name, n_live_tup AS rows
  FROM pg_stat_user_tables
  WHERE schemaname = 'public' AND relname NOT IN ('spatial_ref_sys')
  ORDER BY n_live_tup DESC;"

# Summary
echo ""
echo "========================================="
echo "  Pipeline Complete"
echo "  Total: $TOTAL | Success: $SUCCESS | Failed: ${#FAILED[@]}"
if [ ${#FAILED[@]} -gt 0 ]; then
  echo "  Failed prefectures: ${FAILED[*]}"
fi
echo "========================================="
