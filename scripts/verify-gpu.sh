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
cargo build --locked --features gpu --tests --message-format=json > target/gpu-test-artifacts.jsonl
# Run the test executables, not Cargo itself, under sanitizer.
python3 - <<'PY'
import json
import subprocess
from pathlib import Path
artifacts = [json.loads(line) for line in Path('target/gpu-test-artifacts.jsonl').read_text().splitlines() if line.startswith('{')]
executables = {a['executable'] for a in artifacts if a.get('reason') == 'compiler-artifact'
               and a.get('executable') and a['profile']['test']
               and a['target']['name'] in ('gudra', 'gpu_jacobi')}
assert len(executables) == 2, executables
for exe in sorted(executables):
    for tool in ('memcheck', 'initcheck'):
        subprocess.run(['compute-sanitizer', '--tool', tool, '--error-exitcode', '99', exe, '--test-threads=1'], check=True)
PY
