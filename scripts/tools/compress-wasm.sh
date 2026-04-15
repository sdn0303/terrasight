#!/usr/bin/env bash
set -euo pipefail

WASM_DIR="${1:-services/frontend/public/wasm}"

if [ ! -d "$WASM_DIR" ]; then
  echo "Error: $WASM_DIR does not exist" >&2
  exit 1
fi

echo "=== Compressing WASM files ==="
compressed=0
for f in "$WASM_DIR"/*.wasm "$WASM_DIR"/*.js; do
  [ -f "$f" ] || continue
  gzip -k -9 -f "$f"
  orig=$(wc -c < "$f")
  gz=$(wc -c < "$f.gz")
  pct=$(( (orig - gz) * 100 / orig ))
  echo "  $(basename "$f"): $orig -> $gz bytes (${pct}% savings)"
  compressed=$((compressed + 1))
done

echo "=== Done: $compressed files compressed ==="
