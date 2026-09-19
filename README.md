# Gudra

A Rust crate for one weighted five-point Jacobi step on a two-dimensional grid
with a one-cell halo. The GPU implementation uses cuTile and `f32` arithmetic.

Private source and output owners prevent safe callers from aliasing immutable
inputs with mutable output. The caller supplies halo values; Gudra does not
manage boundary conditions or implement an iterative solver.

**Status:** experimental. CPU tests pass. GPU compilation and execution still
require validation on the supported CUDA environment.

## Quick start

Rust is pinned to 1.89.0. CUDA is optional for CPU development:

```sh
cargo build --locked
cargo test --locked
```

On a configured NVIDIA/CUDA system:

```sh
cargo run --locked --features gpu --example poisson_step
```

The example performs one update on a 17x19 interior and compares every output
cell with the CPU reference. See the [setup guide](docs/setup.md) for prerequisites.

## Operation

```text
candidate = 0.25 * (north + south + east + west - h_squared * rhs)
output    = (1 - omega) * center + omega * candidate
```

`Shape2D` validates dimensions; `JacobiParams` validates coefficients.
`gpu::Gpu` provides blocking `step` and `step_into`, and an owned
`step_into_async`. Uploads and readback block. See the [API guide](docs/api.md)
for ownership, synchronization, errors, and limits.

## Development

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo doc --locked --no-deps
```

## Layout

- `src/` — library and cuTile kernel
- `tests/` — CPU and GPU integration tests
- `examples/` — one runnable Poisson-step example
- `docs/` — setup and API guides
- `scripts/` — GPU prerequisite checker
