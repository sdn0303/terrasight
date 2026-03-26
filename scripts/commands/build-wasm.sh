#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../services/wasm"

wasm-pack build --target web --out-dir ../../services/frontend/public/wasm --release

SIZE=$(ls -la ../../services/frontend/public/wasm/*.wasm | awk '{print $5}')
echo "WASM built: ${SIZE} bytes"
