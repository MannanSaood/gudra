# Ownership and numerical safety model

Gudra's safe API is designed to exclude simultaneous immutable input and mutable
output access to one device allocation. This is a scoped ownership guarantee,
not a claim that every GPU race, backend failure, or mathematical error is ruled
out. GPU compilation and execution are still unverified on the local Windows
host; see [the results and acceptance gate](verification-results.md).

## What enforces the boundary

| Boundary | Mechanism | Executable evidence |
|---|---|---|
| RHS and output cannot be one allocation in a call | `&Field2D` and `&mut Field2D` remain borrowed through completion | `tests/ui/gpu/alias_rhs.rs`, E0502 |
| Source cannot become output | Distinct `HaloGrid2D` and `Field2D` types; no conversion | `source_as_output.rs`, E0308 |
| Two live mutable output aliases cannot be used | Rust exclusive borrowing | `overlap_mut.rs`, E0499 |
| Async work cannot coexist with reuse of moved output | Future owns the buffers | `reuse_owned.rs`, E0382 |
| Caller cannot clone/import the private allocation | No public tensor, pointer, clone, stream, or view access | `private_storage.rs`, E0616; source inspection |
| Geometry and coefficients are in supported domains | Checked constructors and preflight checks | CPU unit/numerical tests and GPU validation tests |

The actual `step_into` output is internally partitioned into 16x16 tiles. The
public API does not expose a partition/subview constructor, so the meaningful
overlap attempt is two live mutable borrows of the whole field. Disjointness of
the *internal* partition, masking of partial tiles, generated launches, device
completion, and storage reclamation remain cuTile/CUDA obligations. UI tests
cannot prove them.

Run `python3 scripts/check-ui.py --gpu` after installing the pinned build
prerequisites. This compiles the real library and then emits metadata for the
fixtures: no GPU execution or allocation occurs. Cargo's exact current metadata
artifact is used. A successful control precedes each rejection lane. The runner
asserts diagnostic error codes and primary spans in the fixture, and rejects
unrelated errors. It never snapshots full compiler wording. Dependency failures
are **blocked checks**, never passing compile-fail cases. The existing rustdoc
compile-fail example is supplementary, not the diagnostic-intent assertion.

`python3 scripts/check-ui.py` runs the CPU API's control and alias rejection
without CUDA. That CPU rejection does not prove the GPU facade's behavior.

## Escape hatches and trusted code

Gudra forbids `unsafe` in its own crate and exposes no unsafe import or raw-device
pointer API. A caller can still violate Rust's rules using unsafe forged
references, bitwise owner duplication, raw pointers, transmutation, or external
CUDA/FFI operations outside this interface. Such code forfeits the proof; `unsafe`
is not permission to make invalid references. Direct use of backend tensors and
launch APIs is also outside Gudra's private-owner guarantee. `mem::forget` is
safe Rust: forgetting a future or owner leaks resources, but does not create a
usable alias. Dependency internals, proc macros, native code, JIT, driver, and
hardware are trusted here, not independently verified.

## Completion and error states

| Operation/result | Publicly accessible state |
|---|---|
| Bad dimensions, coefficients, upload length | Explicit error before allocation/submission at that API boundary |
| Bad shape/session/metadata, invalid field | Borrowed output unchanged; no kernel submitted |
| Bind or view creation fails before execution | No new output write submitted |
| Borrowed execution succeeds | Output ready only after the sync terminal succeeds |
| Borrowed execution errors or unwinds | Output invalidated; readback and use as RHS/output return `InvalidBuffer`; shape and drop remain available |
| Allocating `step` fails | No partial output returned |
| Owned async validation/execution fails | Consumed owners are not returned |
| Async unpolled drop | No kernel submitted |
| Async drop after submission | Dependency attempts stream drain and can block; does not cancel the kernel |
| Readback fails | Consuming readback returns no field or partial host vector |

The private completion guard marks output uncertain before entering backend
execution and clears that state only on success. CPU tests inject an error and
an unwind; GPU unit tests inject that state and exercise every public data path.
These are deterministic state tests, not simulated driver-fault recovery.

A driver/context error can invalidate the whole session. In particular, earlier
dependency analysis did not establish retention of async outer owners when
stream drain fails. No recovery, retry, or lifetime guarantee under that failure
is claimed. Invalidating a field prevents presenting partial numerical results;
it does not repair an unsuccessful device drain. Production fault recovery needs
a separate dependency audit and isolated fault-injection environment.

## Numerical contract and limits

`src/reference.rs` is the straightforward CPU weighted-Jacobi oracle: row-major
source `[height+2,width+2]`, interior RHS/output `[height,width]`, one-cell halo,
rows southward and columns eastward. The four halo corners are unused. The caller
supplies halo values and is responsible for freshness and boundary conditions.

```text
candidate = 0.25 * (north + south + east + west - h_squared * rhs)
output = (1 - omega) * center + omega * candidate
```

Both axes must be positive. Halo addition, element products, and byte products
are checked; bytes fit `isize`, axes/strides/counts fit positive `i32`, and each
16-cell tile axis has at most 65,535 blocks. Wrong lengths and equal-area but
different extents fail explicitly. Sessions cannot be mixed, even at one ordinal.

`omega` can be any finite `f32`, including zero, negative and greater-than-one
values. `h_squared` must be finite and nonnegative (negative zero is accepted).
There is no convergence promise. Tensor NaNs/infinities are accepted rather than
repaired or rejected. Zero coefficients do not suppress their arithmetic effects.
Overflow during floating arithmetic is a numerical value, not `ShapeOverflow`.

For bounded finite fixtures, require
`abs(actual - oracle) <= 1e-5 + 1e-5 * abs(oracle)` for **every** element.
The comparison also requires finite actual values when the oracle is finite.
Expected NaNs require NaNs, without payload/sign assertions; expected infinities
require the same sign. This tolerance accommodates ordinary GPU rounding and
contraction; it is not a universal error bound for arbitrary magnitudes,
cancellation, accumulated iterations, or overflow-sensitive reassociation.
Signed-zero bit equality and subnormal relative accuracy are not promised.
Extreme coefficient fixtures avoid ambiguous cancellation and include explicit
overflow cases. Expanding the supported accuracy envelope needs new evidence.

The oracle is checked independently using literal 1x1/2x3 answers, an `f64`
window-based calculation, and a manufactured quadratic solution. GPU tests
check square, rectangular, narrow, partial-tile and 257x263 (289-tile) domains.
Each production directional view is separately copied using asymmetric exact
sentinels. Merely comparing Jacobi sums would let a north/south swap pass.

## Non-guarantees and negative control

This work does not establish convergence, solver accuracy, valid/fresh halos,
bounded JIT latency, allocation success, deadlock freedom, cancellation latency,
or absence of races in arbitrary other kernels or foreign unsafe code. Stress
repetitions sample schedules; they do not exhaust schedules. First-poll drop tests
can finish immediately and are not proof of forced-pending cancellation.

The [CUDA C++ fixture](../fixtures/README.md) intentionally updates one global
allocation in place. Its race need not reproduce deterministically. NVIDIA's
[current Racecheck documentation](https://docs.nvidia.com/compute-sanitizer/ComputeSanitizer/index.html#what-is-racecheck)
limits detection to on-chip shared-memory accesses (accessed 2026-09-19); it does
not generally detect this global-memory race. Neither a clean Racecheck report
nor repeated matching numbers proves the fixture safe. This educational negative
control is not a CVE or a claim of a distinct security impact.
