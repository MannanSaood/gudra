# One weighted Jacobi step

Gudra implements the canonical operation on row-major `f32` data:

```text
candidate = 0.25 * (north + south + east + west - h_squared * rhs)
output    = (1 - omega) * center + omega * candidate
```

`Shape2D::new(height, width)` describes the interior. Source storage is exactly
`[height+2, width+2]`; RHS and output are exactly `[height,width]`. Row increases
southward and column eastward. Supply every halo cell, including unused corners.
Interior edge cells update from their adjacent supplied halo values. No boundary
condition, fresh halo, convergence criterion, or solver iteration is inferred.

## Basic use

Enable the optional `gpu` feature using the [pinned setup](setup.md).
The [complete example](../examples/poisson_step.rs) fills asymmetric, nonzero
halos and a varying RHS for a 17x19 interior and checks every output cell.

```rust,no_run
use gudra::{gpu::Gpu, JacobiParams, Shape2D};

fn main() -> gudra::Result<()> {
    let shape = Shape2D::new(17, 19)?;
    let params = JacobiParams::new(2.0 / 3.0, 0.01)?;
    let gpu = Gpu::new(0)?;
    let source = gpu.upload_halo(shape, vec![1.0; shape.haloed_len()])?;
    let rhs = gpu.upload_field(shape, vec![0.0; shape.interior_len()])?;
    let output = gpu.step(&source, &rhs, params)?;
    let values = output.into_host()?;
    assert_eq!(values.len(), 323);
    Ok(())
}
```

`reference::jacobi_into` exposes the independent scalar CPU oracle without CUDA.
It checks all lengths before writing and uses ordinary bounds-checked indexing.
Small finite test fixtures compare with `abs(gpu-cpu) <= 1e-5 + 1e-5*abs(cpu)`.
This tolerance is not an error bound for arbitrary data. Input NaNs/infinities
and arithmetic overflow propagate under floating-point arithmetic; there is
no coefficient-zero shortcut or data repair.

## Ownership and completion

| API | Allocation and synchronization |
|---|---|
| `Gpu::new` | Creates a device/context and private stream; all buffers retain the session |
| `upload_halo`, `upload_field` | Consume a host vector, allocate fresh device storage, copy and synchronize |
| `zeros` | Allocate and initialize an interior field, then synchronize |
| `step` | Validate before allocation, allocate/initialize output, execute and synchronize; may wait separately for initialization and execution |
| `step_into` | Borrow inputs and exclusively borrow initialized output through synchronized return; no new grid-sized allocation or host transfer |
| `step_into_async` | Consume all three owners, validate immediately, submit on first poll, return owners only after completion |
| `Field2D::into_host` | Consume the field, allocate host storage and synchronously read back without an extra device copy |
| `shape` | Return checked metadata without synchronization |

JIT, small view metadata, and backend bookkeeping can allocate. All operations
on one `Gpu` share an ordered stream, and completion/drop may wait for other
submitted work on it. No foreign tensors, streams, pointers, graph operations,
mutable views, or storage clones are exposed. A distinct `Gpu` instance is a
distinct session even on the same device ordinal.

The source role cannot be used as output. Rust also rejects simultaneous shared
and mutable borrows of the RHS. A complete compile-fail fixture is attached to
`Gpu::step_into` in rustdoc; GPU-feature doctests must run to validate it.

For async use, first obtain an initialized output with `gpu.zeros(shape)?`:

```rust,ignore
let job = gpu.step_into_async(source, rhs, output, params)?;
// Do unrelated host work. Submission is lazy until polling/awaiting job.
let gudra::gpu::StepBuffers { source, rhs, output } = job.await?;
```

The future implements `Send + 'static` and retains its session even if `Gpu` is
dropped. An unpolled drop submits no kernel; a submitted drop may synchronously
drain the stream. Forgetting the future leaks its retained owners. Validation
errors, execution errors, and cancellation consume the async buffers without
returning them. Prefer blocking borrowed calls when preserving inputs on errors
matters. First poll may JIT synchronously; there is no bounded latency promise.

## Errors and limits

The public CUDA-free `Error` enum distinguishes empty interiors, overflow,
supported-domain limits, wrong lengths, mismatched 2D shapes, wrong sessions,
invalid coefficients, and backend failures. `Display` retains the operation
label and original backend diagnostic. Backend text is not a stable protocol.

Both axes must be positive. Checked arithmetic covers halo dimensions, both
element counts, and byte counts. Bytes fit `isize`; axes, strides, and both
element counts fit positive `i32`. Neither tiled axis exceeds 65,535 blocks.
`1x1`, narrow, rectangular, and partial-tile shapes are valid. `omega` must be
finite; `h_squared` must be finite and nonnegative. Both may be zero, and no
convergence interval is imposed on `omega`.

Validation leaves borrowed output untouched. Execution failure can leave it
partially updated; there is no rollback or retry. Driver/context fault recovery
is outside this contract. In particular, the pinned async dependency's failure
to drain does not establish retention of the outer owned frame; that failure
path requires further dependency review. The memory-access claim depends on cuTile/CUDA and
does not cover compiler, driver, hardware, or external unsafe-code faults.

## Verification status

CPU checks pass locally. GPU compilation stops in `cuda-bindings` because no
CUDA Toolkit is installed, before type-checking this facade or generated
launcher. GPU numerical, compile-fail, and async tests are supplied but remain
unexecuted. See the [setup guide](setup.md#validation-status) for validation status.

Generate public API documentation with `cargo doc --locked --no-default-features
--no-deps`, or `cargo doc --locked --features gpu --no-deps` in the supported CUDA
environment. The latter includes the GPU surface without requiring a GPU launch.
