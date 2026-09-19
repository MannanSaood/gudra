# Readback lifetime repair: verification pending

Outcome: **blocked verification**, with a candidate repair implemented. The
original report has not yet been shown absent on the patched GPU build.

## Evidence and boundary

The user-supplied [pre-fix log](evidence/chat05-gpu-before-readback-fix.log)
records five successful process repetitions, followed by 21 Memcheck
use-after-free reports. Original SHA-256:
`D9D51D987768AA6CB97FA78154EC181861EACF3DBA4A9B9365934E4CA79476F9`.
The first copy/free stacks occur at lines 153-213. The error summary is at 1443.
These are remote observations on Arch/CUDA 13.4, not a local reproduction.
The stored log normalizes line endings and removes trailing whitespace; the hash
above identifies the original attachment.

All reports pass through `Field2D::into_host`. In pinned cuTile 0.3.1:

1. `ToHostVec for Tensor` (`tensor.rs:1249`) consumes the tensor into an Arc.
2. `CopyDeviceToHostVec::execute(self)` (`api.rs:328-352`) queues the copy and
   drops its Arc when execution returns.
3. `DeviceOp::sync_on` (`cuda-async/device_operation.rs:428-435`) synchronizes
   only after `execute` returns.
4. `DeviceBuffer::drop` (`cuda-async/device_buffer.rs:128-146`) enqueues free on
   a separate deallocator stream.

The last device owner can therefore enqueue free before the copy stream's
completion barrier. The trace directly shows this destruction chain. No hostile
input is needed: even a one-element field reaches it through ordinary safe API
use. No distinct security exploit or CVE claim is made.

NVIDIA's [Memcheck stream-ordered race documentation](https://docs.nvidia.com/compute-sanitizer/ComputeSanitizer/index.html#stream-ordered-race-detection)
describes lifetime checks for stream-ordered allocations (accessed 2026-09-19).
This is Memcheck evidence, separate from Racecheck's shared-memory scope.

## Candidate repair and compatibility

`src/gpu/cutile_impl.rs` now retains a local `Arc<Tensor<f32>>`, constructs the
copy from `&Arc`, saves the synchronized result, and explicitly drops the owner
after the terminal call. This is the sole public GPU readback boundary, covering
uploaded fields, zero fields, blocking and owned-async step outputs. Upload
already retains its host owner through its copy terminal.

The public field remains consumed, there is no public alias or extra device copy,
the session remains alive, and error labels/invalidated-field rejection remain
unchanged. No dependency pins, unsafe code, numerical expression, or sanitizer
suppression are introduced. An independent source investigation confirmed the
boundary before editing.

A separate read-only review found no concrete normal-path bypass or regression
in the candidate. Neither review substitutes for a GPU sanitizer rerun.

This repairs retention through the normal synchronization path. A failed stream
drain, driver/context fault, or backend unwind still does not establish device
quiescence; the existing unsupported fault-recovery contract remains. Retaining
through terminal return is not a proof of fault recovery.

## Regression and local checks

`readback_retains_allocation_until_copy_completes` in `tests/gpu_jacobi.rs`
exercises repeated uploads/readbacks and zeros at 1x1, 2x3, 17x19, 33x47 and
257x263. It also reads a field on another thread after dropping its executor.
The original directional/library tests remain unchanged. Ordinary data assertions
are a compatibility control; the regression's memory-lifetime verdict requires
Memcheck with stream-ordered race tracking enabled.

Local checks: formatting, CPU Clippy, diff checks, Python/Bash syntax, 18 CPU
tests plus one doctest, and both CPU UI fixtures passed. Tests used Rust 1.89
with the previously documented installed Cargo workaround; linting used 1.90.
`cargo check --locked --offline --features gpu --all-targets` remains blocked
in cuda-bindings by missing CUDA Toolkit, before patched GPU code type-checking.
No clean sanitizer result or post-patch GPU behavior is claimed.

## Rerun on the friend's machine

Keep the working Arch environment exports. Pull the patch, then run:

```bash
git pull --ff-only
(
  set -euo pipefail
  git rev-parse HEAD
  cargo clippy --locked --features gpu --all-targets -- -D warnings
  python3 scripts/check-ui.py --gpu
  for trial in {1..5}; do
    echo "GPU repetition $trial/5"
    CUDA_ASYNC_SPIN_BUDGET_US=0 cargo test --locked --features gpu --lib --test gpu_jacobi -- --test-threads=1
  done
  python3 scripts/check-gpu-sanitizers.py
) 2>&1 | tee gpu-readback-fix.log
```

The new runner builds fresh test binaries and selects exact executable paths from
Cargo JSON, avoiding stale hardcoded hashes. It runs both library and integration
executables under memcheck and initcheck, enables stream-ordered race tracking,
and fails on sanitizer errors with exit code 99. It does not enforce Ubuntu
package names or silently skip failures. The strict pinned runner delegates its
sanitizer stage to the same script.

Closure requires successful updated GPU tests and all four sanitizer invocations
with zero errors. Send the complete log even on failure. Do not disable tracking
or suppress these reports to obtain a pass.
