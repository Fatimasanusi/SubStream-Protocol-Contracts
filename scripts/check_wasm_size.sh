#!/bin/bash
# Post-build WASM size assertion.
# Fails with exit code 1 if any .wasm file exceeds MAX_SIZE_BYTES.

set -euo pipefail

MAX_SIZE_BYTES=${MAX_WASM_SIZE:-102400}  # 100 KiB default; override via env
WASM_DIR="${1:-target/wasm32v1-none/release}"
FAILED=0

if [ ! -d "$WASM_DIR" ]; then
  echo "ERROR: WASM directory not found: $WASM_DIR" >&2
  exit 1
fi

shopt -s nullglob
WASM_FILES=("$WASM_DIR"/*.wasm)

if [ ${#WASM_FILES[@]} -eq 0 ]; then
  echo "ERROR: No .wasm files found in $WASM_DIR" >&2
  exit 1
fi

echo "WASM size check (limit: ${MAX_SIZE_BYTES} bytes / $(( MAX_SIZE_BYTES / 1024 )) KiB)"
echo "---"

for f in "${WASM_FILES[@]}"; do
  size=$(wc -c < "$f")
  name=$(basename "$f")
  if [ "$size" -gt "$MAX_SIZE_BYTES" ]; then
    echo "FAIL  $name — ${size} bytes (exceeds limit by $(( size - MAX_SIZE_BYTES )) bytes)"
    FAILED=1
  else
    echo "OK    $name — ${size} bytes ($(( MAX_SIZE_BYTES - size )) bytes under limit)"
  fi
done

echo "---"
if [ "$FAILED" -ne 0 ]; then
  echo "Size assertion FAILED. Reduce binary size or raise MAX_WASM_SIZE." >&2
  exit 1
fi

echo "Size assertion PASSED."
