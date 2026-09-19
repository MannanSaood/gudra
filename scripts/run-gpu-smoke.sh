#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd -- "$script_dir/.." && pwd)"

"$script_dir/check-gpu-env.sh"
cd "$repo_dir"

export CUTILE_JIT_TIMING=1
cargo run --locked --features gpu --example vector_add -- "$@"
