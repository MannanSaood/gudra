# Gudra

**Weighted Jacobi stencils on NVIDIA GPUs, with Rust ownership at the API boundary.**

Gudra is a Rust library for a weighted five-point Jacobi update on a two-dimensional
grid. It uses [cuTile](https://github.com/NVlabs/cutile-rs) for GPU execution and
provides an independent CPU reference implementation for validation. Data is
row-major `f32`, with an explicit one-cell halo around the input grid.

[Quick start](#quick-start) · [API guide](docs/api.md) · [Setup](docs/setup.md) ·
[Safety model](docs/safety-model.md) · [Verification results](docs/verification-results.md)

> **Status:** Experimental and distributed from source, with crate publishing
> disabled. Recorded GPU results apply to the revisions and hardware
> listed in the verification report; production release validation remains open.

## Features

- **Checked grid metadata:** dimensions, halo sizes, element counts, and byte counts
  are validated before use.
- **Separate input and output ownership:** private device storage and distinct
  buffer roles prevent safe callers from creating overlapping mutable views.
- **Blocking and asynchronous execution:** allocate a result with `step`, reuse an
  output with `step_into`, or transfer ownership into `step_into_async`.
- **Explicit boundary data:** callers supply the halo and right-hand side for
  predictable stencil behavior.
- **CUDA-free development:** the default build includes the scalar reference,
  shape validation, and CPU tests without compiling the GPU dependencies.

Gudra performs one update. Applications provide boundary conditions, refresh halos,
and control iteration and convergence. Multi-GPU execution and peer transport are
not part of the current public API.

## Quick start

Clone the repository and run the CPU tests:

```sh
git clone https://github.com/MannanSaood/gudra.git
cd gudra
cargo build --locked
cargo test --locked
```

The repository pins Rust **1.89.0** and commits `Cargo.lock` for reproducible
dependency resolution. Use a Rust installation managed by `rustup`.

### Run on a GPU

Enable the optional `gpu` feature on a configured Linux NVIDIA system. The
[setup guide](docs/setup.md) documents the target environment, CUDA toolkit,
driver, GPU, and Clang prerequisites. The GPU backend pins cuTile **0.3.1**.

```sh
./scripts/check-gpu-env.sh
cargo run --locked --features gpu --example poisson_step
```

The example updates a **17 × 19** interior with nonzero, asymmetric halo values
and compares all **323** output cells with the CPU reference. It exits with a
nonzero status if validation fails. The first GPU operation may compile the
kernel, so initial execution can take longer than later calls.

Use a trusted, isolated development environment for GPU execution. Shared GPU
services need application-level resource limits and a separately validated
isolation policy; Rust buffer ownership alone does not provide tenant isolation.

## API example

With the `gpu` feature enabled:

```rust,no_run
use gudra::{gpu::Gpu, JacobiParams, Shape2D};

fn main() -> gudra::Result<()> {
    let shape = Shape2D::new(17, 19)?;
    let params = JacobiParams::new(2.0 / 3.0, 0.01)?;
    let gpu = Gpu::new(0)?;

    // The source includes a one-cell halo on every side.
    let source = gpu.upload_halo(shape, vec![1.0; shape.haloed_len()])?;
    let rhs = gpu.upload_field(shape, vec![0.0; shape.interior_len()])?;

    let output = gpu.step(&source, &rhs, params)?;
    let values = output.into_host()?;
    assert_eq!(values.len(), 323);
    Ok(())
}
```

Uploads, `step`, and readback in this example block until their work completes.
For buffer reuse, asynchronous ownership, cancellation, and error behavior, see
the [API guide](docs/api.md). The [complete example](examples/poisson_step.rs)
also demonstrates validation against `reference::jacobi_into`.

## Numerical operation

For each interior cell:

```text
candidate = 0.25 * (north + south + east + west - h_squared * rhs)
output    = (1 - omega) * center + omega * candidate
```

`Shape2D::new(height, width)` describes the interior. Source storage has
`(height + 2) × (width + 2)` elements; RHS and output each have `height × width`
elements. Rows increase southward and columns eastward.

`JacobiParams` requires finite `omega` and finite, nonnegative `h_squared`.
It does not impose a convergence interval on `omega`. Input NaNs, infinities,
and floating-point overflow can propagate; callers remain responsible for the
numerical suitability of their data and solver.

## Validation and development

Run the CPU development checks without a CUDA installation:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --release
python3 scripts/check-ui.py
cargo doc --locked --no-deps
```

On the configured GPU environment, run the numerical tests and example:

```sh
cargo test --locked --features gpu --test gpu_jacobi -- --test-threads=1
cargo run --locked --features gpu --example poisson_step
```

The [test strategy](docs/test-strategy.md) covers compile-fail ownership checks,
GPU repetitions, and Compute Sanitizer checks. The [verification report](docs/verification-results.md)
records commands, revisions, environments, and results. Passing numerical tests
does not by itself establish memory safety or isolation; the
[safety model](docs/safety-model.md) describes the dependency boundary and the
limits of the guarantees.

## Repository guide

| Path | Contents |
|---|---|
| [`src/`](src/) | Public API, CPU reference, and cuTile kernel |
| [`tests/`](tests/) | Numerical, GPU, and compile-fail ownership checks |
| [`examples/`](examples/) | Runnable Poisson-step example |
| [`docs/`](docs/) | Setup, API, safety, and verification documentation |
| [`scripts/`](scripts/) | Environment checks and verification tooling |
