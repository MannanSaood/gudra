# Development setup

## CPU development

Install Rust 1.89.0 with formatting and linting tools:

```sh
rustup toolchain install 1.89.0 --profile minimal --component rustfmt --component clippy
cargo build --locked
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
```

The default feature set does not compile CUDA dependencies. `Cargo.lock` is
committed to keep development dependencies reproducible.

## GPU environment

The project pins the following validation environment:

| Component | Version or requirement |
|---|---|
| OS | Linux x86_64, Ubuntu 24.04 |
| Rust | 1.89.0 |
| cuTile | 0.3.1 |
| CUDA Toolkit | 13.3.0, full development toolkit |
| NVIDIA driver | R610 or newer |
| GPU | NVIDIA compute capability 8.0+ |
| Clang | clang-18 and libclang-18-dev |

The dependency's CUDA floor is documented in the
[cuTile 0.3.1 release](https://github.com/NVlabs/cutile-rs/releases/tag/v0.3.1).
The table above is the project's pinned target; the full GPU build/run remains
unverified. Consult NVIDIA's [CUDA installation guide](https://docs.nvidia.com/cuda/archive/13.3.0/cuda-installation-guide-linux/index.html)
for driver and toolkit installation.

### Native setup

Install the full CUDA 13.3 toolkit and Clang dependencies, then set:

```sh
export CUDA_TOOLKIT_PATH=/usr/local/cuda-13.3
export PATH="$CUDA_TOOLKIT_PATH/bin:$PATH"
export LIBCLANG_PATH=/usr/lib/llvm-18/lib
./scripts/check-gpu-env.sh
```

The environment checker reports missing prerequisites and exits nonzero.

## Build and run

With the toolkit installed:

```sh
cargo check --locked --features gpu --all-targets
cargo clippy --locked --features gpu --all-targets -- -D warnings
cargo build --locked --features gpu --all-targets
cargo doc --locked --features gpu --no-deps
cargo test --locked --features gpu --doc
```

With supported GPU hardware:

```sh
cargo test --locked --features gpu --test gpu_jacobi -- --test-threads=1
CUDA_ASYNC_SPIN_BUDGET_US=0 cargo test --locked --features gpu --test gpu_jacobi -- --test-threads=1
cargo run --locked --features gpu --example poisson_step
```

The Poisson example checks 323 output cells against the CPU reference and exits
with code 2 on failure. Explicit GPU tests fail when prerequisites are missing;
they do not silently skip. First launch may compile the kernel with `tileiras`.

## Troubleshooting

| Failure | Check |
|---|---|
| CUDA toolkit not found during cuda-bindings build | Install full toolkit and set CUDA_TOOLKIT_PATH |
| tileiras missing or incompatible | Use the pinned toolkit's bin directory |
| libclang or stddef.h missing | Install clang-18 and libclang-18-dev; set LIBCLANG_PATH |

## Validation status

CPU formatting, linting, build, 9 tests, and one doctest pass after cleanup. Initial
CPU build/tests/docs also passed with Rust 1.89's compiler.
GPU checking stopped in the dependency build because the development machine
lacked CUDA headers, before the GPU facade was type-checked. GPU numerical,
aliasing doctest, and async lifecycle behavior remain unverified. The commands
above are the required checks on a supported system.
