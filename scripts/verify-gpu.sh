#!/usr/bin/env bash
# Required GPU gate: fail closed on any compiler, test, or sanitizer error.
set -euo pipefail
cd "$(dirname "$0")/.."
bash scripts/check-gpu-env.sh
cargo check --locked --features gpu --all-targets
cargo clippy --locked --features gpu --all-targets -- -D warnings
python3 scripts/check-ui.py --gpu
cargo test --locked --features gpu --doc
# Unit lane includes the probe of each actual cuTile directional view and the
# public rejection of an invalidated field. Integration lane checks numerics.
for trial in 1 2 3 4 5; do
    printf 'GPU repetition %s/5\n' "$trial"
    CUDA_ASYNC_SPIN_BUDGET_US=0 cargo test --locked --features gpu --lib --test gpu_jacobi -- --test-threads=1
done
python3 scripts/check-gpu-sanitizers.py
