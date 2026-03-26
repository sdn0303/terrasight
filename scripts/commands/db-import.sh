#!/usr/bin/env bash
# Import GeoJSON data into PostGIS using the Python import tool.
# Usage: ./scripts/commands/db-import.sh [--dataset NAME] [--dry-run]
# Requires: uv (for Python dependency management)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Get DATABASE_URL from docker compose environment
export DATABASE_URL="postgresql://app:${DB_PASSWORD:-devpass}@localhost:5432/realestate"

echo "=== Importing GeoJSON data ==="
cd "$ROOT"
uv run scripts/tools/import_geojson.py "$@"
echo "=== Import complete ==="
