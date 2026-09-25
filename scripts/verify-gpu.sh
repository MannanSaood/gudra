#!/usr/bin/env bash
# Required GPU gate: fail closed on any compiler, test, or sanitizer error.
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "${1:-}" != "--private-worker" ]]; then
    exec python3 scripts/run-private-gpu.py -- bash scripts/verify-gpu.sh --private-worker
fi
python3 scripts/verify-gpu-candidate.py --disposable-fault-tests
