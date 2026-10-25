# Release checklist — 2026-10-12

Decision: **HOLD**. This is a local review candidate, not an approved release.
Owner: repository owner under the recorded solo-maintainer policy. All open
items are due before publication; no independent approval is claimed.

- [x] Owner approved `gudra` as final name on 2026-10-12; repository exists at
  https://github.com/MannanSaood/gudra. Registry name lookup must be refreshed
  immediately before any separately authorized publication; availability is not
  reservation or ownership.
- [x] MIT package expression/license; backend Apache-2.0 licenses and source
  notices retained; NOTICE explains packaging and native non-redistribution.
- [x] Repository/homepage/README, keywords/categories and default docs metadata
  added; no unavailable docs.rs page is advertised.
- [x] Fresh registry API and sparse index lookups both returned 404 for `gudra`
  on 2026-10-12; GitHub API confirmed the public repository and main branch.
  See `docs/evidence/chat12/name-availability.json`. Name is not reserved.
- [x] Candidate version `0.1.0-rc.1` selected for source-only GitHub distribution;
  crates.io is outside this release channel. Publication remains unauthorized.
- [x] Complete pinned Rust 1.89.0 format/lint/test/docs and default package checks
  passed on the reviewed source snapshot. See `docs/evidence/chat12/remediation/`;
  these supersede the earlier mixed-toolchain limitation. Final hosted SHA still
  requires its own checks.
- [ ] All format, lint, debug/release, UI, doctest, docs, package, advisory,
  license/source/bans, dependency, vendor and secret checks pass on final SHA.
- [ ] Current RustSec database identity/date recorded; offline cached success
  alone is insufficient. Gitleaks scans current source and available Git history.
- [ ] Isolated Linux launcher/JIT separate-UID checks and hosted CI green.
- [ ] GPU build/lint/docs/doctests/UI, repeated numerical/readback/budget/batch/
  cancellation tests, sticky-fault fail-stop and memcheck/initcheck on declared
  isolated hardware match the exact release tree. Use `scripts/verify-gpu.sh`;
  do not run destructive fault tests on a shared or sensitive worker.
- [x] Fresh static security review covers the maintained source-release diff,
  with no reportable findings and all 21 authoritative items closed. Scan
  `4b7f9cf4-d530-4ae0-9da4-5dc6e0b7020d`, completed 2026-10-25, is bound to the
  restored source snapshot; it is not a new GPU or hosted execution claim.
- [ ] `scripts/check-release-gate.py` passes for the final source digest;
  `docs/security/release-gate.json` is the authoritative reference. Never replace
  its digest solely to turn a failing check green.
- [ ] Inspect actual `.crate` and source archive inventories, size and paths;
  verify checksums, secret scan, normalized manifest and locked dependency graph.
- [x] Source channel retains all three reviewed patches and Apache-2.0 notices.
  Registry-safe equivalents are not required for this source-only channel. Cargo
  drops root `[patch]` during packaging and downstream use ignores dependency
  patches. Current crates.io release is blocked; `publish = false` stays set.
- [ ] Clean committed source; source/SBOM/toolchain/checksums attested from
  protected main at the approved full SHA; reviewer verifies subject digests.
- [ ] Reconfirm protected main/tags/release environment, required checks,
  maintainer MFA, minimal ownership and private reporting on GitHub. Local YAML
  does not prove administrative settings. Preserve solo-maintainer governance.
- [ ] Owner explicitly authorizes exact version, tag, artifacts and channel.
  Technical approval and workflow dispatch are not publication authorization.
- [ ] Retain prior verified artifacts/evidence; rollback and private incident
  procedures tested by a tabletop review; see `operations.md`.

Optional unverified GPU preview labeling does not waive the security gate.
No tag, release, crates.io upload, push, credential or remote-setting change was
performed by this release preparation.
