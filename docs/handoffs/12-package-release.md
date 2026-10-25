# Chat 12 release engineering handoff

## Resumed closure update — 2026-10-25

The earlier status below is historical. See
[current status](../release/status-2026-10-25.md) and
[current source review reference](../security/release-review-2026-10-25.json).
The complete pinned Rust 1.89 CPU/package checks passed, exact public-checksum
scanner exceptions and negative controls pass, fresh dependency/scanner/Linux
launcher checks pass, and the maintained-source security diff review completed
with no findings. The release candidate is `0.1.0-rc.1`, source-only, with all
three safety overrides retained; crates.io remains disabled.

The current gate is explicitly HOLD for final immutable GPU execution, protected
hosted CI, owner MFA confirmation, signed provenance and publication authorization.
September's approved gate is retained as `release-gate-2026-09-26.json`; it was
not repurposed as approval for the new candidate. No public publication occurred.

Date: 2026-10-12. Checkout: `gudra`; `gudra-public` untouched.
Recommendation: **HOLD for public release; source review candidate only, GPU
unverified for this final tree.** No tag, push, GitHub release, crates.io publish,
yank, message, credential or remote repository setting change occurred.

Owner approved retaining `gudra`. Both crates.io API and sparse index returned
404 on this date; the GitHub API confirmed MannanSaood/gudra is public with main
as default branch. Availability is not reservation. Keep the existing development
version `0.0.0` and `publish = false`; proposed first version `0.1.0-rc.1` needs
maintainer approval and complete gates.

## Delivered

- README mandatory release gates, Cargo metadata/docs defaults and package
  allowlist; MIT/Apache NOTICE, changelog and draft release notes.
- Release checklist, supported configuration/MSRV/CUDA/version policy,
  checksum/Sigstore plan and rollback/yank/security-response procedure in
  `docs/release/`.
- Fail-closed release evidence workflow now checks the bound security gate and
  repeats UI/secrets/dependency/package checks. CPU CI verifies packaged contents
  and denies rustdoc warnings. Existing SHA action pins, hosted PR isolation,
  no credentials, restricted OIDC job and release environment remain in place.
- Curated deterministic source archive helper retaining vendor patches and
  licenses; Cargo archive inspector rejects generated/native/credential/private
  path and oversized files. Historical evidence/benchmark dumps and standalone
  halo prototypes are intentionally excluded from source candidates.
- Local command output, name/dependency/normalized-manifest/archive review and
  scanner findings in `docs/evidence/chat12/`. Unsigned archives, SBOM, manifest,
  source inventory and SHA256SUMS are under `target/chat12-final/`; these ignored
  local outputs must be retained separately if needed. The inventory's tree
  digest identifies a dirty snapshot, not a reviewed Git commit.

## Verification and limits

CPU format and denied-warning all-target Clippy passed with stable 1.90 tools.
Build/debug/release/doctest/UI checks passed with Rust/rustdoc 1.89 and Cargo
1.90. This is a disclosed workaround: pinned 1.89 installation lacks Cargo and
Clippy locally. Direct docs/package checks could not traverse inaccessible
historical Chat 10 scratch directories; the curated copy has passed docs with
warnings denied and verified default-feature Cargo packaging. Final artifact
checks are recorded in `artifact-results.json`, including actual inventories and
separate shipped-source/crate secret scans. No unreadable scratch directory is
selected for shipping. Full-source scan coverage cannot be inferred for those
directories.

Vendor and executable dependency inventory passed. Fresh online cargo-deny
advisory/license/bans/sources check passed using a workspace-local database;
its exact commit/date is in `advisory-db-identity.txt`. There are no ignored
advisories. The mixed local toolchain does not establish clean pinned Linux CI.

The redacted full-source Gitleaks scan failed with 22 generic-api-key matches in
historical checksum manifests; structural triage is in `secret-triage.json`.
No scanner suppressions were added and no secret values were printed. The
available 43-commit history scan passed. The full-source gate stays open until
reviewed false-positive handling or cleanup is agreed; a clean shipped-artifact
scan is a narrower result and does not override that gate.

GPU build, tests and UI were attempted and blocked at missing CUDA, before
changed facade validation. Local GPU execution, Linux helper/separate-UID
execution and sanitizer/fault checks were not performed. Returned Chat 08/09
hardware records remain historical, revision-scoped evidence. The standalone
multi-GPU prototype is excluded and cannot be marketed as supported.

Cargo normalized the package without root patches. The source graph has three
reviewed path overrides and otherwise locked crates.io sources; no Git source
is allowed. Default package compilation does not establish GPU safety. Both
Cargo packaging and downstream dependency use lose root patches: publishing
this GPU feature would silently drop private-JIT/completion safety fixes.
Obtain reviewed registry equivalents or design/review a deliberately CPU-only
package before considering crates.io. Do not remove the patches for convenience.

## Final security gate and closure work

Authoritative historical gate: `docs/security/release-gate.json`; enforcement:
`python3 scripts/check-release-gate.py`. The current source differs and the guard
fails as intended. No historical digest/status was rewritten to claim approval.
No high/critical issue was confirmed by the base review; that statement does
not certify the post-base runtime delta. A current review is still mandatory.

Before release: review the final runtime/dependency/workflow delta, triage full
secret-scan findings, choose a version, commit a clean candidate, run exact pinned
CPU/hosted checks and isolated exact-tree GPU suite, close the digest-bound
security gate with actual evidence, reverify GitHub protections/MFA/private
reporting/minimal ownership, and generate/verify signed provenance. Source review
and technical gate passage still require a separate explicit owner instruction
for any public publication. Unverified-preview labeling does not waive security.
