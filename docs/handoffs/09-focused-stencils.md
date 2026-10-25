# Chat 09 handoff: focused stencil library

Date: 2026-10-03. Workspace: `gudra`, the active implementation checkout.
Status: Chat 09 runtime acceptance PASS on the returned Arch experimental
configuration. CPU validation also passed; this is not publication approval.
Historical OPEN/NOT RUN statements below describe the state before returned
v2 evidence and are superseded for the tested configuration by the final section.
The pre-existing Chat 08 changes were preserved. No commit, push, release or
external message was made. The `gudra-public` checkout was not modified.

Selected only the five-point Laplacian, reusing the MVP's 2D f32 geometry and
ownership model. Its concrete APIs are `LaplacianParams::new(h_squared)`,
`reference::laplacian_into(shape, halo, output, params)`, and
`Gpu::laplacian_into(source, output, params)` (blocking/preallocated output).
No new dependency, handwritten unsafe code, macro/DSL, generic precision policy,
or dimensionality framework was introduced.

Artifacts:

- `src/params.rs`, `src/reference.rs`, `src/gpu/kernels.rs`: coefficients and arithmetic.
- `src/gpu/cutile_impl.rs`: existing directional views, partition, budget and
  completion reused; shared validation extracted with optional RHS.
- `tests/laplacian.rs`: five CPU tests covering analytic curvature, affine and
  constant fields, one-cell asymmetric halo, nonfinites, independent f64 windows,
  thin/partial-tile grids, spacing rejection and atomic length validation.
- `tests/gpu_laplacian.rs`: bounded seeded oracle matrix across twelve shapes
  and three spacings, nonfinite classification, wrong extents and foreign sessions.
- UI positive controls plus CPU alias and GPU wrong-output-role rejection fixtures.
- `examples/laplacian_curvature.rs`, `examples/poisson_residual.rs`: common
  reference/GPU comparison helper; CPU paths passed locally. The residual is an
  explicitly separate host reduction, not a kernel safety guarantee.
- `examples/stencil_bench.rs`: seven medians of 100 preallocated calls on three
  shapes and spacings; GPU feature adds warmed blocking timings and readback checks.
- [Operator matrix/conventions](../operators.md), [ADR 009](../adr/009-focused-operators.md).
- [Local evidence](../evidence/chat09/): debug/release logs, Clippy, rustdoc,
  ownership controls, examples, benchmark JSON, formatting and GPU blocker.

Local validation: Windows x86_64, stable Rust 1.90.0 (the pinned 1.89 toolchain
is incomplete locally). Debug and release each passed 28 unit/integration tests
and one doc test. CPU Clippy all-targets with denied warnings, rustdoc, three UI
controls, formatting and whitespace passed. Both examples passed, including
quadratic curvature 6 and Poisson max residual 0.051231384 after 500 steps.
Benchmarks recorded in the operator matrix are one shared-host CPU run, without
an asserted performance threshold. Independent f64 fixtures are validation only.

The actual GPU all-targets check failed in cuda-bindings before compiling the
new implementation: no CUDA 13+ toolkit was found. Thus new GPU compilation,
UI, oracle tests, examples, JIT, sanitizers and timings are NOT RUN. Prior Chat 08
GPU results belong to an older candidate and do not validate this change. The
new method is available in source but is not yet a validated GPU configuration.

On the configured Linux CUDA worker, prepare a clean candidate using the existing
[friend runbook](08-friend-gpu-runbook.md), then use the updated evidence runner:

```sh
python3 scripts/run-private-gpu.py -- python3 scripts/verify-gpu-candidate.py --arch-experimental
```

Use `--arch-experimental` only for that lane; omit it for the configured pinned
lane. Do not enable disposable device-fault tests on a shared worker. The runner
now includes `gpu_laplacian` in the repeated test lane and the sanitizer discovery
requires all three test executables (`gudra`, `gpu_jacobi`, `gpu_laplacian`).
It runs all three new examples in release GPU mode. Existing timestep/cancellation
runs retain zero spin budget; new examples/benchmark unset that budget and label
the evidence as default. Kernel launch and synchronization time are included;
JIT, upload, initialization and readback are excluded from GPU timing.

Return the clean revision, source manifest, toolchain/device identity, full log
and `target/gpu-candidate-evidence.json`. Verify that the manifest matches the
candidate, fix any compile/numerical/sanitizer errors, and record all nine GPU
benchmark rows before claiming GPU acceptance. Consider default and zero-spin
comparisons only if the new blocking timings motivate them. CPU performance is
sampled rather than guaranteed for every spacing or Shape2D size. No additional
operators should be selected from these CPU timings alone.

The remaining acceptance requirement is numerical and performance evidence for
the new GPU configuration, including code-generation/build diagnostics and
sanitizer execution. This handoff makes that gap explicit; acceptance is not PASS.


## First returned GPU attempt and correction

The user supplied console output from the Chat 09 worker. Rust 1.89.0,
CUDA/tileiras 13.4.92, driver 615.71.09, RTX 5060 Laptop GPU (CC 12.0),
Compute Sanitizer 2026.3. The backend lifecycle/fault-policy/release tests passed,
and `cargo check --locked --features gpu --all-targets` passed. GPU Clippy then
failed on `many_single_char_names` in the new Laplacian entry's five load bindings.
The runner stopped there: GPU numerical tests, ownership UI, sanitizer and new
benchmark evidence were not produced. The printed child-worker aborts were
expected passing fault-policy/release checks, not the stopping failure.

Correction: rename the five bindings to center_values/north_values/south_values/
east_values/west_values. Arithmetic and partitioning are unchanged. Local CPU
Clippy and formatting passed after the change; GPU Clippy still requires the
worker run. Reissued source bundle: `gudra-chat09-source-v2.zip`. Use a fresh
extraction, verify hashes, create a local clean validation commit, and rerun the
normal evidence command. Do not claim acceptance from the initial partial log.

Archived user-supplied output: `docs/evidence/chat09/first-gpu-attempt.log`.
No returned manifest or structured JSON has been verified for this attempt.


## Returned v2 results: runtime acceptance PASS

The user supplied `chat09-gpu-results.tar.gz` for revision
`28a9100000f5ab38154410ebdb4e8c327869e82f`. Its 378-entry source manifest equals
the sent v2 manifest; all sent ZIP entries and current local source files matched
those hashes before this documentation update. The lock hash matches, the
revision file equals the evidence revision, and every recorded command output
appears in the full log. These are internally consistent user-supplied results;
the remote Git tree was not independently retrieved or attested.

All 26 commands passed: actual GPU compilation, Clippy with denied warnings,
rustdoc, 12 ownership UI controls, four doc tests, five repetitions of 14 library
+ 13 Jacobi + 3 Laplacian tests, backend lifecycle/release/drop tests, and all
examples. All three test executables passed memcheck and initcheck with six
zero-error summaries. The Laplacian's twelve-shape/three-spacing oracle matrix,
nonfinite classification and atomic validation tests passed each repetition.

Hardware: RTX 5060 Laptop GPU, CC 12.0, driver 615.71.09. Rust/Cargo 1.89.0,
CUDA/tileiras 13.4.92, Compute Sanitizer 2026.3, Linux x86_64. Lane:
`arch-experimental`, not the pinned release lane. Run started 2026-10-03
16:42 UTC (22:12 Asia/Calcutta).

Both new release examples passed GPU/reference assertions: quadratic curvature
6 in all 323 cells and Poisson residual 0.051231384 after 500 Jacobi steps.
All nine GPU benchmark rows and matching worker CPU rows were captured with
Laplacian spin budget unset (default). At 17x19 the GPU is launch dominated:
7.094–7.113 us versus CPU 0.190–0.267 us. At 256x256 it takes 8.171–8.262 us
versus CPU 13.812–14.004 us; at 257x263, 8.497–8.648 us versus CPU
13.794–13.896 us. Larger sampled grids show about 1.60–1.71x lower latency
than the worker's scalar CPU oracle. This is one warmed, preallocated blocking
run excluding transfers/JIT, not an optimized CPU comparison or an end-to-end
GPU speedup. CPU model and power/clock state were not recorded.

The old Poisson timestep comparison remains a zero-spin measurement:
819 us blocking / 1310 us owned async, ratio 1.600. It is separate from the
new Laplacian benchmark and does not close the earlier default-spin timestep
follow-up.

Chat 09's configuration-scoped gate is now met: existing safe views/partitions,
explicit numerical conventions, tests, performance samples, examples and handoff.
No additional operators or precision/dimension abstractions are warranted from
these results. The deliberate sticky device-fault lane was not run in this
archive; publication/release and broader hardware support are not established.
No further friend run is needed for the scoped Chat 09 normal-path gate.

Evidence: [review and hashes](../evidence/chat09/returned-v2/review.json),
[structured results](../evidence/chat09/returned-v2/gpu-candidate-evidence.json),
[full log](../evidence/chat09/returned-v2/gpu.log),
[benchmark rows](../evidence/chat09/returned-v2/benchmark.json).
