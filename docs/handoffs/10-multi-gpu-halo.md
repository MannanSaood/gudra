# Chat 10 handoff: safe multi-GPU halo model

Date: 2026-10-03. Delivered and locally verified: hardware-independent checked
protocol prototype and standalone native adapter. Native multi-GPU acceptance
is pending hardware. Progression to Chat 11 is approved; see the disposition below.

The prerequisites were checked before implementation. Recorded Chat 08
single-GPU evidence passes; the completed September 27 follow-up reports no
confirmed vulnerability, and current SECURITY.md reports no confirmed high or
critical findings. The local display inventory reports only Intel UHD 620 and
no NVIDIA CLI/compiler tools. No assumption of two installed GPUs was made.

Artifacts:

- `prototypes/halo`: standalone Rust crate, private completion bindings, consuming
  ownership gates, CPU stencil, overlap/reference scheduling and fault injection.
- `docs/multi-gpu-halo.md`: architecture/protocol and native CUDA contract.
- `docs/handoffs/10-halo-failure-injection.md`: results and coverage limits.
- `docs/evidence/chat10`: debug/release/Clippy outputs, simulated timeline CSV,
  environment record, source SHA-256 manifest and threat-model location pointer.
- Scoped updated threat model saved through the Codex Security artifact store;
  its exact path is in `docs/evidence/chat10/threat-model-location.json`.

Verified: eight unit tests per profile, 35,328 exact reference comparisons per
profile, ownership compile-fail controls, valid API positive control, all-target
Clippy, formatting and whitespace. This crate is separate from Gudra's native
runtime and adds no CUDA dependency. Existing Chat 08/09 edits were preserved.

Acceptance disposition: stale/mismatched halo epochs cannot authorize boundary
compute through the model's safe API; numerical equality and atomic error
publication pass. Actual P2P, native identity/lifetime enforcement and real
overlap remain unverified. No GPU performance claim is made.

Next owner steps:

1. Implement an owned native CUDA adapter against the protocol contract, including
   directed P2P capability/enablement, bounded staging fallback and actual context,
   allocation-generation, stream and event binding.
2. Use a trusted worker with at least two discovered NVIDIA GPUs; record UUIDs,
   topology, directed peer capabilities, driver/toolchain and exact clean revision.
3. Run numerical/stress/fault/ownership/sanitizer checks, including partial failure,
   event reuse, unprovable drain and supervisor timeout. Native faults must dispose
   of the worker without prematurely freeing in-flight owners.
4. Collect real timelines and equal-work end-to-end costs including communication,
   synchronization and fallback. Keep deterministic overlap-disabled reference.
5. Re-run the security review before public release, covering P2P/context authority,
   raw handles, any new IPC, untrusted dimensions and service-wide resource limits.

No commit, push, external message or release was performed. Publication remains
disabled. The previous release-gate record is historical and does not approve
this new prototype or a future native adapter.

## Development progression disposition — 2026-10-12

The owner explicitly removed two-GPU hardware availability/validation as a gate
for advancing to the next development stage. Chat 10 development is complete
with native hardware validation deferred; proceed to Chat 11 documentation and
community work. There is no need to wait for a two-GPU volunteer to start it.

This disposition does not turn pending tests into passes. CUDA build, two-GPU
correctness, P2P/staging, sanitizers, actual overlap and driver-fault disposal
remain open contributor follow-ups. Native acceptance, production release and
GPU performance claims still require their evidence. The historical security
release gate and runtime epoch/identity/error checks are unchanged.

The [community validation issue](https://github.com/MannanSaood/gudra/issues/3)
is open on GitHub. Its text is retained in
`docs/handoffs/10-community-validation-issue.md`. Offer contributors the existing
source-only `gudra-halo-native-20261003.zip` package; its README reflects the
October 3 review snapshot. This later documentation disposition does not modify
that archive, executable sources, or sealed security evidence.

## Native adapter follow-up — 2026-10-03

The user authorized adapter implementation, available checks and security review
while looking for a two-GPU worker. `prototypes/halo-native` now supplies an
isolated native CUDA worker with actual P2P copies, portable pinned-host staging,
private checked native completion bindings, generation gates, interior overlap,
reference mode, bounded polling, owner-retaining drain and process fail-stop.
This is a standalone C++/CUDA executable, not a Rust FFI/cuTile integration.

Local verification passes: 4,608 exact coordinator/oracle generations, 21 failure/
admission cases, concurrent simulations, three fail-stop child checks, and five
supervisor assertions. The Windows runner builds current source and records its
binary digest. See `docs/evidence/chat10/native-final/halo-w41inmrl/evidence.json`.

The hardware runner and [friend runbook](../../prototypes/halo-native/README.md)
are ready. On a trusted Linux machine with two selected NVIDIA GPUs, CUDA tools,
Compute Sanitizer and Nsight Systems, run `python3 run_checks.py --profile`.
An explicitly selected `--driver-fault` additionally launches a GPU 1 trap in a
disposable child last; use only an isolated worker with no valuable concurrent
GPU work. Return the entire evidence directory. No hardware was contacted or
tools installed in this task.

CUDA build/type checking, real P2P/staging, GPU numerics, native sanitizers,
driver-fault disposal and actual overlap profiles remain pending. No GPU speedup,
production safety, shared-tenant isolation or release approval is claimed.
The first static review found no confirmed vulnerability and identified three
robustness gaps. These are now fixed: observation poisoning, stale Windows binary
provenance, and full-log memory loading. A second review is bound to the frozen
native candidate; the generated report location is recorded in
`docs/evidence/chat10/native-review-location.json`.
