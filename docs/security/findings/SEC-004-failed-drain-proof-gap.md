# Security Finding

## Metadata

- ID: SEC-004
- Title: Failed stream drain does not establish outer-owner retention
- Status: suspected safety failure; confirmed proof gap
- Severity: informational (no validated exploit severity)
- Affected versions/components: cuda-async 0.3.1 DeviceFuture; Gudra owned async facade
- Reporter/date: independent local audit, 2026-09-19
- Owner/deadline: runtime safety maintainer, 2026-09-26

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
