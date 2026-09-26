# Development setup

## CPU development

Install Rust 1.89.0 with formatting and linting tools:

```sh
rustup toolchain install 1.89.0 --profile minimal --component rustfmt --component clippy
cargo build --locked
cargo test --locked
python3 scripts/check-ui.py
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
```

The default feature set does not compile CUDA dependencies. The separate UI
runner requires Python 3 and no third-party Python packages. `Cargo.lock` is
committed to keep development dependencies reproducible.

## GPU environment

The project pins the following validation environment:

| Component | Version or requirement |
|---|---|
| OS | Linux x86_64 |
| Rust | 1.89.0 |
| cuTile | 0.3.1 |
| CUDA Toolkit | 13.3 or newer within CUDA 13.x; full toolkit; `nvcc` and `tileiras` must match |
| NVIDIA driver | R610 or newer |
| GPU | NVIDIA compute capability 8.0+ |
| Clang | Clang and discoverable libclang 18 or newer |

The dependency's CUDA floor is documented in the
[cuTile 0.3.1 release](https://github.com/NVlabs/cutile-rs/releases/tag/v0.3.1).
The table above is the project's reviewed compatibility range. A prior clean
candidate passed the normal Arch/CUDA 13.4 lane; the final disposable fault lane
must run again on the current revision. Consult NVIDIA's
[CUDA installation guide](https://docs.nvidia.com/cuda/cuda-installation-guide-linux/)
for driver and toolkit installation.

### Native setup

Install a full reviewed CUDA toolkit and Clang dependencies, then set paths for
the selected installation:

```sh
export CUDA_TOOLKIT_PATH=/usr/local/cuda
export PATH="$CUDA_TOOLKIT_PATH/bin:$PATH"
export LIBCLANG_PATH=/path/to/libclang
./scripts/check-gpu-env.sh
```

The environment checker reports missing prerequisites and exits nonzero.

## Build and run

With the toolkit installed:

```sh
cargo check --locked --features gpu --all-targets
python3 scripts/check-ui.py --gpu
cargo clippy --locked --features gpu --all-targets -- -D warnings
cargo build --locked --features gpu --all-targets
cargo doc --locked --features gpu --no-deps
cargo test --locked --features gpu --doc
```

With supported GPU hardware, start runtime commands with private JIT temporary
storage (see [SECURITY.md](../SECURITY.md)):

```sh
python3 scripts/run-private-gpu.py -- cargo test --locked --features gpu --test gpu_jacobi -- --test-threads=1
python3 scripts/run-private-gpu.py -- cargo test --locked --features gpu --lib -- --test-threads=1
CUDA_ASYNC_SPIN_BUDGET_US=0 python3 scripts/run-private-gpu.py -- cargo test --locked --features gpu --test gpu_jacobi -- --test-threads=1
python3 scripts/run-private-gpu.py -- cargo run --locked --features gpu --example poisson_step
```

The Poisson example checks 323 output cells against the CPU reference and exits
with code 2 on failure. Explicit GPU tests fail when prerequisites are missing;
they do not silently skip. First launch may compile the kernel with `tileiras`.

## Troubleshooting

| Failure | Check |
|---|---|
| CUDA toolkit not found during cuda-bindings build | Install full toolkit and set CUDA_TOOLKIT_PATH |
| tileiras missing or incompatible | Use the pinned toolkit's bin directory |
| libclang or stddef.h missing | Install Clang/libclang 18 or newer; set LIBCLANG_PATH |

## Validation status

See [verification results](verification-results.md) for current commands and
results, including the local incomplete Rust installation workaround. The GPU
dependency build is blocked by missing CUDA before facade type-checking. Earlier
remote GPU results are recorded separately; final-revision security changes need
fresh numerical, directional, UI, async and resource-budget verification. Run
`bash scripts/verify-gpu.sh` for the complete gate, including library view tests,
repetitions, and memcheck/initcheck; see [the strategy](test-strategy.md).
The runner automatically creates a private Linux JIT temp directory. Use only a
reviewed revision and dependencies on an isolated GPU worker without release or
production credentials; never execute downloaded PR-produced cubins or binaries.
