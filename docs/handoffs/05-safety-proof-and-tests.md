# Chat Handoff

## Chat

- ID and title: Chat 05 — Prove Safety Boundaries and Numerical Correctness
- Date: 2026-09-19
- Repository: `gudra`, base `7166630`, uncommitted Chat 05 working-tree changes

## Current follow-up status

The user supplied GPU verification from Arch Linux / RTX 5060 Laptop / CUDA 13.4.
Initial ownership and numerical checks passed. Subsequent Memcheck found 21
readback use-after-free reports; commit `073f7c6` repaired owner retention through
synchronization. [The post-fix log](../evidence/chat05-gpu-readback-fix.log) identifies
that commit and records all four Memcheck/Initcheck invocations passing with
eight tests and zero errors each. GPU Clippy and the readback regression pass.
The reported defect is resolved on this setup, based on remote user-supplied
evidence. The local Windows machine still has no CUDA runtime verification.

The later [repetition log](../evidence/chat05-gpu-final-repetitions.log) records
five passing repetitions of 8 library + 8 integration tests, including the new
readback regression, plus four zero-error sanitizer summaries. This file omits
the exact commit and sanitizer tool labels; the earlier labeled log establishes
the sanitizer result on `073f7c6` separately. The user subsequently confirmed the
repetition run also used `073f7c6`, before the documentation-only updates. Its
commit attribution is therefore user-confirmed, not an unresolved blocker.

Remaining for a complete release record: final-revision UI/doctest refresh and
optional execution of the
documented negative control.
Driver-fault recovery remains separate work. Proceed to Chat 06 Security Review
with these limits and the [updated results](../verification-results.md).

The original handoff below is retained as historical context; its missing-GPU
status does not describe the later remote evidence.

## Original outcome

Partially complete; acceptance is blocked by missing CUDA build/runtime access.
CPU debug and release suites each pass 18 tests plus one doctest. The CPU public
API UI control compiles and its alias case fails with E0502. Real GPU API fixtures,
numerical/exceptional-value tests, separate directional probes, repeated lifecycle
tests, negative CUDA control, and a fail-closed GPU runner are ready for execution.
The actual GPU facade has still not been type-checked on this host.

Verification found that borrowed execution errors left potentially partial output
available for read/reuse. A private completion guard now invalidates that output
until success; public data paths reject invalidated fields. Error/unwind state
tests pass on CPU. The GPU wiring and dependency fault behavior remain unverified.

## Artifacts

- Existing `src/reference.rs` scalar CPU oracle retained and independently verified.
- `tests/numerical.rs`, `tests/support/mod.rs`: independent f64/literal/manufactured
  oracles, seeded properties, shape/overflow/parameter/length/nonfinite coverage.
- `tests/ui/{cpu,gpu}/*.rs`, `scripts/check-ui.py`: real current library metadata,
  compile-success controls, diagnostic code/fixture-span assertions, no mocks.
- `tests/gpu_jacobi.rs`: 12 shapes through 289 tiles, repeated output reuse,
  unusual coefficients, nonfinite policy, all validation roles, async/drop checks.
- `src/gpu/verification.rs`: separate reads of actual production directional
  views and invalidated field rejection through every public data path.
- `src/completion.rs`, `src/error.rs`, `src/lib.rs`, `src/gpu/cutile_impl.rs`:
  conservative invalidation and `Error::InvalidBuffer`.
- `fixtures/in_place_jacobi.cu`, `fixtures/README.md`: intentionally racy global
  in-place stencil, 20 reset trials, no required nondeterministic outcome.
- `scripts/verify-gpu.sh`: GPU compile/lint/UI/doc tests, five process repetitions,
  and memcheck/initcheck of the actual test executables.
- `docs/safety-model.md`, `docs/test-strategy.md`, `docs/verification-results.md`,
  `docs/evidence/chat05-*.txt`; updated README, API, and setup links.
- Official source checked: [NVIDIA Racecheck documentation](https://docs.nvidia.com/compute-sanitizer/ComputeSanitizer/index.html#what-is-racecheck),
  accessed 2026-09-19. It does not generally detect global-memory races.

## Verification

| Command/check | Result | Evidence |
|---|---|---|
| CPU debug and release tests with Rust 1.89 compiler | Each 18 tests + 1 doctest pass | `docs/evidence/chat05-cpu-*.txt` |
| CPU UI runner | Positive control and E0502 pass | `docs/evidence/chat05-ui-cpu.txt` |
| Formatting, CPU clippy, docs, script syntax, diff whitespace | Pass; lint compiler 1.90 | `docs/verification-results.md` |
| GPU UI runner | Blocked before GPU type-checking by missing CUDA Toolkit | `docs/evidence/chat05-ui-gpu-blocked.txt` |
| GPU numerical/probe/lifecycle/UI diagnostics, sanitizers, CUDA C++ control | Not executed | Full pending gate in `scripts/verify-gpu.sh` and fixture README |

The pinned local Rust installation lacks Cargo. Installed Cargo 1.90 with explicit
Rust 1.89 compiler/rustdoc paths ran CPU evidence. See the exact workaround and
distinguished compiler versions in `docs/verification-results.md`.

## Decisions

- Keep the existing readable CPU oracle and verify it independently; do not clone
  GPU indexing into another reference and call that independent evidence.
- Inspect each directional view: isotropic Jacobi can hide swapped directions.
- Compare bounded finite values with absolute+relative 1e-5 tolerance; separately
  assert NaN/infinity classes. Do not claim this bounds arbitrary cancellation.
- Use deterministic bounded properties without adding dependencies; literal
  regressions should capture future failures before expanding generators.
- Use exact library artifacts and structured compiler diagnostics. No mocks or
  platform failures stand in for GPU ownership proof.
- Invalidate uncertain outputs instead of allowing partial data to appear ready.
  Revisit only with a verified recovery/reinitialization protocol.

## Risks and limitations

- No GPU results: all CUDA-dependent code, including new tests, may reveal real
  composition/compiler issues when the required environment becomes available.
- Output invalidation is not stream-drain fault recovery. The earlier cuTile
  async owner-retention question on failed drain remains open.
- First-poll cancellation need not be pending; forced-pending cancellation,
  forgotten-future leak behavior, driver fault injection, and 32-bit execution
  are not certified. Repeated tests do not exhaust schedules.
- No convergence/halo freshness/arbitrary-data error bound, broad race freedom,
  or security vulnerability/CVE claim is made.

## Open items

- Blocker: supported Linux/CUDA build environment and NVIDIA runtime hardware.
- Follow-up: run full gate, fix actual compiler/test findings, record environment
  identity and outputs, then update the acceptance table.
- Follow-up: dependency-level failed-drain lifetime review and isolated fault tests.
- Questions requiring user direction: none. Do not treat unrun GPU checks as passes.

## Recommended next chat

- **Chat 05 closure on supported hardware (including Chat 02/04's outstanding
  GPU build/JIT gate), then Chat 06 — Security Review.**
- Ready inputs: safety model, test strategy, result logs, GPU runner, UI fixtures,
  numerical/probe tests, negative control, and earlier toolchain/architecture handoffs.
- GPU acceptance remains required before representing the ownership and numerical
  claims as experimentally verified on the supported backend.
