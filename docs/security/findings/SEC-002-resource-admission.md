# Security Finding

## Metadata

- ID: SEC-002
- Title: Add explicit resource admission for applications accepting untrusted requests
- Status: mitigated for logical session admission; GPU wiring execution pending
- Severity: informational (deployment-dependent hardening; potential service availability impact)
- Affected versions/components: baseline 22f7ff1 GPU constructors and owned operations
- Reporter/date: independent local audit, 2026-09-19

## Summary

Representation-valid dimensions formerly permitted a 4 GiB interior allocation (32768x32768), with no aggregate buffer/job budget. This is not an unconditional vulnerability in an in-process scientific library. It matters when a service passes lower-trust dimensions or request volume into the library without independent quotas.

## Threat model

- Attacker capability: dimensions/data/request rate exposed by a hypothetical embedding service.
- Required access: that service and absent ingress/admission limits.
- Trust boundary crossed: one client consuming resources shared with others.
- Assets affected: host/GPU availability.

## Evidence and reproduction

`Shape2D::new(32768,32768)` succeeds without allocating and represents 4 GiB of interior f32 data. The former `Gpu::zeros` submitted it directly to the allocator. A CPU regression now rejects it through default ResourceLimits without attempting a huge allocation.

## Impact

Conditional service denial of service; no concrete deployed endpoint was provided. A local user exhausting their own GPU is not counted as a security vulnerability.

## Root cause

Backend representation limits were serving as the only dimensions/queue limits. They are not service resource policy.

## Remediation

- Immediate containment: validate shape before host input allocation and bound request/session/process resources.
- Code fix: configurable per-buffer bytes, total bytes, live-buffer count and operation count; default 256 MiB/1 GiB/64/8. Mutex admission and non-cloneable guards follow owners/futures.
- Regression test: CPU huge-request/overflow/count/queue/concurrency/unwind/forget cases; GPU unpolled-drop/full-release/successful-async tests supplied.
- Documentation: logical charges exclude host arrays, JIT/native overhead, allocator/deallocator backlog and failed-drain physical reclamation.

## Verification

CPU debug/release tests pass. New GPU integration test is blocked locally before facade type-checking by missing CUDA. Source review found no normal-path guard defect; GATE-GPU remains required. No retry loop was introduced.

## Disclosure

Hardening record, not a CVE or independently counted vulnerability. No external publication.
