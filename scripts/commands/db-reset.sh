#!/usr/bin/env bash
# Reset database: migrate + seed. Requires running db container.
# Usage: ./scripts/commands/db-reset.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== Resetting database ==="
bash "$SCRIPT_DIR/db-migrate.sh"
bash "$SCRIPT_DIR/db-seed.sh"
echo "=== Database reset complete ==="
