# Chat 08: typed single-GPU timestep runtime

Status: runtime acceptance PASS on the supplied Arch experimental evidence;
security review remains scheduled. This is not publication or release approval.

## Returned GPU evidence — 2026-09-26

The friend returned clean candidate `9ec1a2e4ea794d2db99dc55899e0f42735869c7d`.
All 24 recorded commands passed, including GPU build/Clippy/docs, 10 UI controls,
five repetitions of 14 library and 13 integration tests, 125-step numerical
comparisons, actual Pending cancellation/drop on another thread, resource/error
tests, backend drop tests, and the disposable driver-fault fail-stop test. The
CUDA 719 abort in the fault log is the expected child-worker outcome, and the
parent test passed. Both executables passed memcheck and initcheck with zero
reported errors. The Poisson example also passed its numerical assertions.

The observed hardware is **RTX 5060 Laptop GPU**, compute capability 12.0,
driver 615.71.09, CUDA/tileiras 13.4.59, Rust 1.89.0, Compute Sanitizer 2026.3.
This corrects the planned RTX 5070 description. The runner explicitly labels
this `arch-experimental`; do not relabel it as a pinned-lane run.

The returned manifest equals the sent ZIP manifest. All 280 listed local files
matched their hashes before this documentation-only evidence update, and the
reported Cargo.lock hash matches. Every command output in the JSON occurs in the
fault log. These are internally consistent user-supplied results, not remote
attestation of the tested Git tree. The original ZIP remains unchanged.

| 125 steps, 17x19, median of five | Blocking | Owned async | Async / blocking |
|---|---:|---:|---:|
| Normal run | 835 us | 1289 us | 1.543 |
| Fault-suite run | 798 us | 1274 us | 1.596 |

Async is 54–60% slower for this small scalar-tile workload. It provides owned
asynchronous control, not a measured throughput improvement. Timings exclude
construction/readback and first-launch JIT. They do not compare graph replay or
the old 16x16 interior-only kernel, and do not establish general GPU throughput.

Evidence and SHA-256 records: [review.json](../evidence/chat08/review.json),
[full command evidence](../evidence/chat08/gpu-candidate-evidence.json),
[normal log](../evidence/chat08/normal.log), [fault log](../evidence/chat08/fault.log).
The security follow-up remains scheduled; these test results are not that review.
The earlier OPEN/NOT RUN statements below describe the pre-handoff local state
and are superseded by this returned evidence for the tested candidate.

## Original implementation handoff (historical)

Workspace: `gudra` (the active implementation checkout). `gudra-public` and its
published release were not edited. This candidate has not been committed,
pushed, or released by this task.

## Delivered

- `src/gpu/timesteps.rs`: checked construction, two private haloed allocations,
  immutable RHS, blocking/owned async batches, completion swaps, consuming
  observation, resource admission, count overflow and discard-on-error semantics.
- `src/gpu/kernels.rs`: fixed-boundary, full-grid scalar-tile Jacobi kernel.
- `docs/timestep-runtime.md`: API usage, state/lifetime diagram, compile-time vs
  runtime guarantees, drop/error/unknown state, streams and Send/Sync analysis.
- ADRs 008a/008b/008c: owned lazy async, double buffering, deferred graphs.
- GPU tests: 125-step oracle comparison, boundary bits, odd/even swaps, zero-step,
  owner/executor lifetime, another-thread completion and pending drop, admission,
  overflow, backend rank rejection and reservation release. New compile-fail
  fixtures reject early reuse/private storage access and positive controls assert
  Send+Sync. Existing backend fault-policy/drop seams remain required.
- `examples/poisson_timesteps.rs`: 125-step Poisson run, 25-step observations,
  then median-of-five direct blocking vs owned async timing with the same kernel.
- Existing GPU evidence runner now runs that example after sanitizer checks.

## Local evidence (2026-09-26)

Windows x86_64; installed stable Rust 1.90.0. The pinned 1.89.0 toolchain lacks
its cargo executable here, so local commands used `+stable` explicitly.

| Check | Result |
|---|---|
| CPU tests, debug | 23 unit/integration + 1 doc test passed |
| CPU tests, release | 23 unit/integration + 1 doc test passed |
| CPU Clippy, all targets, deny warnings | passed |
| CPU rustdoc | passed |
| CPU UI positive/negative controls | 2 passed |
| Formatting and diff whitespace | passed |
| GPU check | BLOCKED in cuda-bindings build: CUDA toolkit absent |
| GPU UI/new runtime tests/kernel JIT/sanitizers | NOT RUN |
| Actual timing values | NOT MEASURED |

CPU results do not compile or validate the feature-gated timestep implementation.
No mock backend results are substituted. WSL Ubuntu also lacks the configured
Rust/CUDA tools; the user will give the candidate to a friend with RTX 5070 and
Arch Linux. Use [the friend runbook](08-friend-gpu-runbook.md).

## Acceptance gate

Ownership is enforced by the API design, but actual GPU compile controls remain
unexecuted. The 125-step GPU comparison, pending-drop test, driver-fault
compatibility and sanitizer suite must pass before claiming acceptance. On any
recoverable runtime error the entire state is consumed; unprovable completion
uses the existing backend fail-stop instead of returning unknown storage.

The scalar-tile kernel is deliberately a baseline, limited to <=65,535 blocks
per haloed axis; no performance benefit is claimed. Optimize only after the
provided same-kernel comparison and a separate 16x16/halo-update baseline have
been measured. Graph replay is deferred, with parity/address/shape rules in ADR.

No handwritten unsafe, new FFI binding, dependency update or new cache was added.
The new kernel and readback path use the existing unsafe/FFI/JIT infrastructure,
so a one-time read-only security follow-up is scheduled for 2026-09-27 10:00
Asia/Calcutta in this Codex task: `one-time-timestep-security-review`. It will
report findings here. The review has not run or approved the change.

## Next handoff

Send the generated source ZIP and its SHA-256 file to the friend. They should
return `target/gpu-candidate-evidence.json` even on failure, plus the full log,
manifest, and hash. The runbook creates a new local Git snapshot so the existing
clean-candidate runner has an exact revision. It does not push anything. Treat
this as a new candidate, not the prior release-gate PASS. Fix any compilation,
GPU numerical, sanitizer or timing-harness failure before declaring completion.


## Additional returned run — 2026-10-03

The supplied clean Arch experimental candidate `6cb183ad730cabee8cc8f8b572a1c5cfd38b04c3`
passed all 24 commands, including the repeated 125-step suite, ownership controls,
cancellation, disposable device fault and four zero-error sanitizer checks.
Every JSON command output matches the supplied fault log. The manifest is
identical to the original 280-file bundle; no source delta is described by it.
The remote Git tree was not independently retrieved.

Normal-run median blocking/async times for 125 steps were 832/1291 us (1.552x);
the fault-suite run recorded 801/1268 us (1.583x). Hardware remains the RTX 5060
Laptop GPU. This reproduces the existing evidence configuration.

The read-only security review completed in this task on September 27 and found
no confirmed security vulnerability in the changed code. It identified that
`verify-gpu-candidate.py` forces `CUDA_ASYNC_SPIN_BUDGET_US=0` before both the
runtime suites and performance example. The new manifest contains the same
runner, so this evidence does not close the default-spin follow-up. The original
54–60% slowdown and this rerun's 55–58% slowdown describe spinning disabled.
The earlier statements that the security review is scheduled are historical;
the review result is recorded here without implying formal release approval.

For a separate default-setting correctness and timing check on the friend's
configured GPU worker, run each as a fresh process outside the full runner:

```bash
env -u CUDA_ASYNC_SPIN_BUDGET_US -u CUDA_ASYNC_HOST_SYNC python3 scripts/run-private-gpu.py -- cargo test --locked --features gpu --test gpu_jacobi timesteps_match_oracle_and_observe_without_races -- --test-threads=1
env -u CUDA_ASYNC_SPIN_BUDGET_US -u CUDA_ASYNC_HOST_SYNC python3 scripts/run-private-gpu.py -- cargo run --locked --release --features gpu --example poisson_timesteps
```

Save command text, outputs, revision and toolchain identity with these runs.
Unsetting the variable around `verify-gpu-candidate.py` is insufficient because
that script sets it to zero again. The forced-Pending cancellation checks remain
covered by the existing zero-spin lane; these additional commands assess the
normal default fast path. No implementation change is required to run them.

Additional evidence: [review and hashes](../evidence/chat08/rerun-2026-10-03/review.json).
