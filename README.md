# Gudra

An in-place Jacobi stencil can silently compute the wrong iteration. Each cell
must read its neighbors from the **old** grid; writing back into that grid lets
later readers see a mixture of old and new values. On a GPU, unsynchronized
neighbor reads and writes can also race. A block-local barrier cannot order
all blocks. A sequential in-place sweep is a different algorithm, not Jacobi.

Gudra makes separate input and output storage part of its Rust API for weighted
five-point Jacobi and a five-point Laplacian on haloed, row-major 2D `f32` grids.
It uses [cuTile Rust](https://github.com/NVlabs/cutile-rs) and supplies a scalar
CPU reference. It is experimental, source-only, and unpublished (`publish = false`).

## What is protected

Through Gudra's safe public GPU API, source owners have a distinct type from
output owners; device storage is private and cannot be cloned/imported. RHS
shared borrows cannot coexist with mutable output borrows of the same field.
Owned async work retains its buffers until completion, so a caller cannot reuse
moved buffers while work is in flight. Internal output partitions are intended
to be disjoint; their implementation depends on cuTile/CUDA and is tested, not
proved by Rust borrowing alone. If completion cannot be proven, the worker
aborts before storage is released or reused.

This guarantee assumes valid Rust callers and the reviewed backend. It does
**not** establish halo freshness, boundary mathematics, convergence, universal
race freedom, driver recovery, compiler/hardware correctness, or tenant isolation.
Unsafe forged aliases and external CUDA/FFI access are outside the boundary.
Cancellation can block and discard state; it does not undo a kernel.
Read the [exact safety model](docs/safety-model.md),
[security review](docs/security/audit-report.md), and [SECURITY.md](SECURITY.md).

## Try it without CUDA

Use Git, rustup with Rust 1.89.0 (Cargo, rustfmt, Clippy), a native linker/build
toolchain, and Python 3 for ownership checks. Internet access is needed for the
first toolchain/dependency download. The lockfile and vendored overrides must stay
with the source. These commands assume the documented candidate is present in
your checkout; confirm its revision before evaluating it.

```sh
git clone https://github.com/MannanSaood/gudra.git
cd gudra
rustup toolchain install 1.89.0 --profile minimal --component rustfmt --component clippy
cargo build --locked
cargo test --locked
cargo run --locked --example cpu_step
python3 scripts/check-ui.py
```

The minimal [runnable example](examples/cpu_step.rs) reads asymmetric neighbors:

```rust
use gudra::{reference::jacobi_into, JacobiParams, Shape2D};
let shape = Shape2D::new(1, 1)?;
let source = [0.0, 1.0, 0.0, 6.0, 4.0, 2.0, 0.0, 3.0, 0.0];
let rhs = [0.0];
let mut output = [0.0];
jacobi_into(shape, &source, &rhs, &mut output,
            JacobiParams::new(0.5, 1.0)?)?;
assert_eq!(output, [3.5]);
# Ok::<(), gudra::Error>(())
```

Expected terminal output: `old center=4 new center=3.5`. No GPU fallback is
implied: this explicitly calls the CPU reference. See [setup and diagnostics](docs/setup.md),
[troubleshooting](docs/troubleshooting.md), and the
[performed clean-source walkthrough](docs/new-user-walkthrough.md) for measured
results. The [current release status](docs/release/status-2026-10-25.md) records
the subsequent complete pinned-toolchain verification.

## Aliasing fails before execution

This CPU example deliberately fails with E0502:

```rust,compile_fail
use gudra::{reference::jacobi_into, JacobiParams, Shape2D};
let shape = Shape2D::new(1, 1).unwrap();
let mut rhs = vec![0.0];
jacobi_into(shape, &[1.0; 9], &rhs, &mut rhs,
            JacobiParams::new(0.5, 1.0).unwrap()).unwrap();
```

The GPU equivalent `gpu.step_into(&source, &rhs, &mut rhs, params)` also violates
borrowing. A `HaloGrid2D` cannot be passed as a mutable `Field2D` output.
The [real API UI fixtures](tests/ui/) check positive controls and expected
diagnostic codes; missing dependencies count as blocked checks. CPU rejection
alone does not validate the GPU facade. The deliberately wrong
[CUDA fixture](fixtures/README.md) illustrates the original failure mode.

## Run on a GPU

GPU execution requires the [reviewed Linux CUDA environment](docs/setup.md):
Rust 1.89.0, cuTile 0.3.1, full CUDA 13.3+ within 13.x with matching nvcc/tileiras,
R610+ driver, CC 8.0+ GPU, and Clang/libclang 18+. These are the project's
admission requirements, not a test matrix covering every such configuration.
Recorded recent runs use Arch Linux and an RTX 5060 Laptop GPU.

```sh
bash scripts/check-gpu-env.sh
python3 scripts/run-private-gpu.py -- cargo run --locked --features gpu --example poisson_step
```

The GPU example checks all 323 cells of a 17x19 update against the CPU reference
and exits nonzero on disagreement. First use may JIT-compile. Start with an
isolated trusted development worker; consult SECURITY.md for deployment limits.

## Numerical correctness and measurements

```text
candidate = 0.25 * (north + south + east + west - h_squared * rhs)
output    = (1 - omega) * center + omega * candidate
```

The source has `(height+2)*(width+2)` cells; RHS/output have `height*width`.
Rows increase southward, columns eastward; corners are initialized but unused.
Callers provide boundary samples. Fixed-Dirichlet timestep batches copy the
boundary unchanged. No other automatic boundary policy is implemented.

CPU tests include analytic and independently indexed f64 fixtures. GPU tests
compare bounded finite fixtures with `abs(gpu-cpu) <= 1e-5 + 1e-5*abs(cpu)` and
check nonfinite classifications, thin grids, and partial tiles. This is a fixture
tolerance, not a universal scientific error bound. Numerical tests, compile-fail
checks, security review, and sanitizers each cover different obligations.

The [evidence index](docs/validation-status.md) distinguishes historical release
checks, returned single-GPU results, local CPU checks, and pending two-GPU work.
A sampled warmed Laplacian call on 256x256 took 8.171-8.262 us on GPU versus
13.812-14.004 us in the worker's scalar CPU reference. On 17x19 the GPU was slower:
7.094-7.113 us versus 0.190-0.267 us. See
[methodology, all measurements, and limits](docs/benchmarks.md). Transfers and
JIT are excluded; these are not end-to-end or optimized-CPU speedup claims.

## Explore and contribute

- [API guide](docs/api.md), [operators](docs/operators.md), [timestep ownership](docs/timestep-runtime.md)
- [Architecture](docs/architecture.md) and [evidence-linked roadmap](docs/roadmap.md)
- [Contribution guide](CONTRIBUTING.md) and [code of conduct](CODE_OF_CONDUCT.md)
- [Feedback questions and drafts](docs/community/posts.md), [feedback log and rubric](docs/community/feedback.md)
- [Private vulnerability reporting](https://github.com/MannanSaood/gudra/security/advisories/new); if unavailable, follow [SECURITY.md](SECURITY.md) without posting details publicly

Multi-GPU execution is a [standalone prototype](docs/multi-gpu-halo.md), not a
public Rust API. Its host tests do not establish CUDA correctness or overlap
performance; [two-GPU validation is open](https://github.com/MannanSaood/gudra/issues/3).
Gudra is MIT licensed; vendored licenses/notices are retained. CUDA components
are not bundled. Community drafts are prepared locally; feedback collection
begins after the maintainer posts them and the referenced candidate is accessible.

## Mandatory release gates

Version `0.1.0-rc.1` is a **source-only, GPU UNVERIFIED preview**.
It is not a verified GPU release.
The current static diff review completed on 2026-10-25 with no reportable
findings; [current status and remaining gates](docs/release/status-2026-10-25.md)
bind that review to the candidate. The owner authorized the explicitly unverified
preview allowed by the acceptance criteria; protected CI and signed provenance
must verify before publication. Crates.io stays disabled because Cargo's packaged
registry GPU dependencies do not retain our patches.
Use the [release checklist](docs/release/checklist.md),
[support/version policy](docs/release/supported-configurations.md),
[draft notes](docs/release/notes.md) and
[provenance/rollback procedures](docs/release/operations.md).

A release requires a clean immutable reviewed SHA, complete pinned-toolchain
format/lint/debug/release/UI/doctest/docs/package checks, fresh advisory and
license/source/dependency policy, vendor integrity and current/history secret
scans, inspected source/package contents, and exact-source GPU evidence on
declared isolated hardware (or explicit unverified preview labeling). It also
requires current security review with no unresolved high/critical finding,
passing `python3 scripts/check-release-gate.py`, verified protected repository
and release settings, checksums/SBOM/provenance, and explicit owner publication
authorization. Unverified labeling cannot waive security approval.
