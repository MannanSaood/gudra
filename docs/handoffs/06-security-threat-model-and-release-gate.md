# Chat 06 handoff: security threat model and release gate

Date: 2026-09-19. Repository: Gudra. Baseline `22f7ff1`; remediation is the commit containing this handoff.

**Release decision: FAIL.** No confirmed critical/high vulnerability. One medium transient JIT isolation finding has a private-temp launch mitigation; direct library callers/upstream compiler still require closure. Failed-drain owner retention, final-revision GPU checks and external release protections remain open.

## Delivered

- SECURITY.md; docs/security/threat-model.md and audit-report.md.
- Four template-based records: SEC-001 medium, SEC-002/003 hardening, SEC-004 proof gap.
- ResourceLimits with 256 MiB/buffer, 1 GiB/session, 64 live buffers and 8 admitted operations; explicit configuration/static shape preflight.
- CPU budget regression tests and strengthened GPU release/cancellation/completion tests.
- Private Linux JIT temporary-directory launcher; GPU verifier enters it automatically.
- Pinned least-privilege hosted CI, cargo-deny, Gitleaks, dependency/native/build/proc-macro inventory and policy checker, Dependabot.
- Dependency unsafe lexical inventory and reached-boundary review; CPU/advisory/license/secret evidence.
- Precisely scoped local follow-up issues with roles/deadlines; no external issue publication.

## Validation

23 CPU tests + one doctest pass in both debug/release; CPU UI, formatting, Clippy, docs and dependency policy pass. RustSec database d5c17953a895cf19e8d3ce66eaa42b6fcfe1fb16 yields advisory/license/bans/source pass. Gitleaks finds zero current/history secrets in the available nine baseline commits. Tests use Rust 1.89 with Cargo 1.90 workaround; lint/format use 1.90.

The GPU build fails before facade type-checking because local CUDA is missing. Historical remote evidence at 073f7c6 is not fresh evidence for these changes. Private-temp behavior needs Linux execution. No driver fault or malicious cubin was exercised.

## Next owner actions

Use docs/security/follow-up-issues.md as the work queue; runtime/GPU/CI/repository-owner roles are proposed and need actual assignment. Target review date is 2026-09-26.

1. Patch/review upstream per-compilation private temp isolation and prove it under a separate UID; authorize private upstream coordination separately.
2. Establish actual owned retention/quarantine on failed drain, or narrow the API/deployment boundary; do not equate invalid output with completed GPU work.
3. On an isolated trusted Linux GPU worker, run `bash scripts/verify-gpu.sh` at the exact final revision, plus forced-pending/fault tests. Record compiler/driver/tool versions and complete logs with sensitive paths/data redacted.
4. Verify hosted CI, private reporting, required reviews/checks, protected tags, maintainer/registry 2FA/ownership, log retention, project license and signed release provenance before enabling publication.

Keep `publish = false`. CPU-only development may continue. Session accounting is logical, not physical-memory or tenant isolation. Persistent cuTile cache remains disabled by Gudra. No additional destructive testing, publication or CVE authority was assumed.

Run `python3 scripts/check-release-gate.py` before considering publication; it
currently fails intentionally and also rejects source/configuration drift from
the recorded review digest.
