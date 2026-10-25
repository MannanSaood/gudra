# Focused operator support

The implementation checkout is named Gudra; this is the HaloSafe-style library
in the prompt. The selected addition is the five-point Laplacian only. See
[ADR 009](adr/009-focused-operators.md) for selection and deferred abstractions.

```rust
use gudra::{reference::laplacian_into, LaplacianParams, Shape2D};
let shape = Shape2D::new(17, 19)?;
let halo = vec![1.0; shape.haloed_len()];
let mut out = vec![0.0; shape.interior_len()];
laplacian_into(shape, &halo, &mut out, LaplacianParams::new(0.25)?)?;
# Ok::<(), gudra::Error>(())
```

With `gpu`, upload with `Gpu::upload_halo`, allocate with `Gpu::zeros`, then call
`gpu.laplacian_into(&source, &mut output, params)` and `output.into_host()`.
The new GPU method is blocking and writes an existing allocation. There is no
Laplacian double-grid or async API.

| Operator/configuration | CPU reference | GPU | Numerical/performance evidence |
|---|---|---|---|
| Weighted Jacobi, 2D f32 compact row-major, supplied halo | existing | existing 16x16 partition | prior Chat 05/08 evidence, scoped to recorded revision |
| Fixed Dirichlet Jacobi batches, 2D f32 | caller iteration | existing scalar-tile runtime | Chat 08 on RTX 5060 Laptop, zero spin budget |
| Laplacian, 2D f32 compact row-major, isotropic spacing, supplied halo | validated locally | validated on recorded Arch configuration | CPU analytic/f64 oracle tests; GPU oracle/sanitizers passed; worker CPU/GPU timings below |
| Heat, gradient/divergence, reaction-diffusion | deferred | deferred | no application/performance evidence |
| 3D, f64, anisotropic spacing, general strided/transposed layouts | no API | no API | deferred |
| Automatic periodic/Neumann/Dirichlet halo management for Laplacian | no API | no API | caller must materialize samples |

Physical source extent is `[height+2,width+2]`, output `[height,width]`, columns
contiguous. Rows increase southward, columns eastward; corners are unused.
Laplacian sign is positive: `L(u) = (north + south + east + west - 4*center)/h^2`.
The implementation adds the neighbors left to right, subtracts `4*center`, then
multiplies by a precomputed f32 reciprocal. It does not divide per cell. The
constructor rejects nonfinite/nonpositive squared spacing and infinite
reciprocals. Fields may contain NaNs or infinities; no coefficient shortcut
suppresses them. Overflow is propagated rather than returned as an error.

Fixture comparison tolerance is `1e-5 + 1e-5*abs(cpu)` for bounded finite values;
nonfinite comparisons classify NaNs and require infinity signs. This does not
bound scientific discretization error or arbitrary cancellation, small-spacing
amplification, overflow, or GPU contraction. Exact dyadic analytic fixtures use
exact CPU values. The independent f64 window oracle is test-only, not an f64 API.

Length checks reject short/long source or output before writes. GPU checks reject
equal-area wrong extents, another executor, unavailable storage and private
layout invariant failures. f64 and halo outputs fail to type-check. Correct-size
samples do not prove halo freshness or boundary mathematics. A flattened array
whose length is correct cannot reveal whether a caller transposed its meaning.

Run both examples without CUDA, or add `--features gpu` to compare against the
new GPU kernel:

```sh
cargo run --locked --release --example laplacian_curvature
cargo run --locked --release --example poisson_residual
cargo run --locked --release --example stencil_bench
```

The curvature example verifies `x*x+2*y*y` gives 6. The Poisson example runs 500
Jacobi steps with fixed zero boundary samples then computes `L(u)-rhs`. Its max
residual is a host reduction separate from output partition safety.

Benchmark: release build, seven samples of 100 calls, median microseconds/call,
preallocated output, warm execution; CPU black_box prevents dead-code removal.
GPU mode excludes upload/initialization/readback/JIT, includes blocking launch
and synchronization, and compares readback to the CPU oracle. This is observed
wall-clock latency, not device event time, bandwidth, or a speedup claim.

2026-10-03 local Windows x86_64, Rust stable 1.90.0:

| Interior | h^2=0.25 | h^2=1 | h^2=4 |
|---|---:|---:|---:|
| 17x19 | 1.440 us | 1.603 us | 1.441 us |
| 256x256 | 65.389 us | 75.666 us | 81.652 us |
| 257x263 | 68.560 us | 75.885 us | 65.033 us |

These are one local run on a shared host; no performance threshold is asserted.
Timings sample configurations, not the entire Shape2D/spacing domain. GPU results
from earlier operators cannot be transferred to this kernel or changed candidate.


## Returned v2 worker evidence

The Chat 09 scoped runtime gate passed on the RTX 5060 Laptop GPU / Arch
experimental lane with Rust 1.89.0, CUDA/tileiras 13.4.92 and driver 615.71.09.
Earlier statements about pending new GPU validation are historical. The returned
manifest matches v2; see the [acceptance handoff](handoffs/09-focused-stencils.md)
for evidence checks and limits. This does not establish a pinned release lane.

Same-worker median us/call, warm preallocated blocking execution with default
spin budget. Each entry is **CPU / GPU**, with seven samples of 100 calls:

| Interior | h^2=0.25 | h^2=1 | h^2=4 |
|---|---:|---:|---:|
| 17x19 | 0.267 / 7.113 | 0.199 / 7.108 | 0.190 / 7.094 |
| 256x256 | 14.004 / 8.171 | 13.812 / 8.258 | 13.963 / 8.262 |
| 257x263 | 13.896 / 8.613 | 13.794 / 8.497 | 13.839 / 8.648 |

The GPU has lower latency on the two larger sampled grids; CPU is faster on the
tiny grid. Compare these same-worker CPU rows, not the earlier Windows timings.
This is a scalar-oracle comparison excluding transfers/JIT; CPU model, power
and clock state were not recorded. No automatic backend choice is introduced.
The five repeated GPU suites, six clean sanitizer runs, and both examples close
the new-operator validation gap for this recorded configuration.
