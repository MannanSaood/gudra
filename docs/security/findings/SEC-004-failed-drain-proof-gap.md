# Security Finding

## Metadata

- ID: SEC-004
- Title: Failed stream drain does not establish outer-owner retention
- Status: suspected safety failure; confirmed proof gap
- Severity: informational (no validated exploit severity)
- Affected versions/components: cuda-async 0.3.1 DeviceFuture; Gudra owned async facade
- Reporter/date: independent local audit, 2026-09-19
- Owner/deadline: repository owner acting as runtime safety maintainer until delegated; before release, progress review 2026-10-02 (accepted 2026-09-25)

## Summary

Normal cancellation drains the stream. On drain failure, the dependency forgets its result tuple; Gudra supplies borrowed child arguments, so that does not retain the outer owned buffers. A live-work use-after-free after a specific CUDA failure has not been demonstrated. Existing docs already acknowledge this gap.

## Threat model

- Attacker capability: no established Gudra input that reliably induces the required live-work fault.
- Required access: device/driver fault with outstanding work plus unsuccessful drain.
- Trust boundary crossed: Rust lifetime model to asynchronous backend reclamation.
- Assets affected: potentially device-memory/result integrity; actual exploit uncertain.

## Evidence and reproduction

`cuda-async-0.3.1/src/device_future.rs:297-323` binds and waits normally; `:336-347` forgets only `result` on wait failure. The outer async frame in `src/gpu/cutile_impl.rs` owns buffers and lends child references. Their drops reach `device_buffer.rs:128-146` and a separate deallocator stream; `cuda-core/src/api.rs:117-122` requires ordering after all uses. No destructive fault injection was attempted.

## Impact

Unproved quiescence/retention means the strongest safety acceptance gate cannot pass. Sticky faults may disable further execution/freeing, so a failed drain alone is insufficient proof of exploitable use-after-free. No current high vulnerability is asserted.

## Root cause

Retaining borrowed results does not retain their owners; numerical invalidation is distinct from device completion.

## Remediation

- Immediate containment: isolate workers, discard process/session on fault; no retry/recovery claim.
- Code fix/follow-up: owned retention through confirmed completion or permanent quarantine, including synchronous terminals and panic paths; alternatively restrict API behind an owned isolated worker.
- Regression test: deterministic fault seams plus isolated GPU completion/deallocation evidence; forced-pending cancellation.
- Documentation: all guarantees stop short of driver/context fault recovery.

## Verification

Normal source paths, retained readback fix and prior normal-path sanitizer logs reviewed. Fault proof and final GPU tests remain pending. This record blocks safety acceptance rather than inflating an uncertain exploit to high severity.

## Disclosure

Local follow-up only. No upstream report, exploit test outside project scope, or CVE request.

## Chat 07 review

Still unresolved, release blocking. An independent review confirmed the proof gap extends beyond async drop: cuda-async device_operation.rs sync_on drops its result when synchronization fails; execute unwind skips synchronization. DeviceFuture consumes its operation before storing a result, so partial submission errors/unwinds lack retention. Backend-local host DMA destinations also prevent a first-party outer guard from proving safety. A reviewed dependency-level ownership/fail-stop redesign and actual fault seam tests are still required. No exploit or severity escalation is asserted. Owner acceptance remains pending; deadline 2026-09-26.

## 2026-09-25 remediation status

Implementation is complete for Gudra's supported public GPU facade at public
candidate `1ee62e582912590ae27af6256b6da9c6262aed1c`. Patched root terminals
retain results and a transferred allocation quarantine until completion is
proven; partial-submit errors synchronize; failed drains/unwinds abort before
owner release. Upload, readback, zero initialization, session retention, and
logical reservations were reviewed together. A first independent review found
an early async quarantine release, which was corrected; final review found no
Gudra-public bypass. Status remains verification pending until the exact
candidate passes the supplied real-device fault and sanitizer lane. This remains
a proof gap under validation, not a confirmed exploit or severity escalation.

The user authorized selection of the accountable role on 2026-09-25. The
repository owner holds the remaining device-validation work until a runtime
safety maintainer explicitly accepts delegation. Fail-stop worker disposal is
the enforced mitigation; release rationale remains that real-device fault and
sanitizer evidence is required before this proof gap can be closed.
