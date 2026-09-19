# Security audit and release gate

**Decision: FAIL for release.** Reviewed 2026-09-19, from baseline `22f7ff122706bb200bf57749fba96d16d0c74654` plus this local remediation. No confirmed critical/high finding was established. One medium dependency isolation finding remains conditionally mitigated, and required GPU/unsafe fault-path evidence is incomplete. Experimental CPU development can continue; this is not approval for production or mutually untrusted GPU tenants.

## Scope and method

Reviewed all 52 baseline first-party tracked files: Rust APIs/kernels/tests/examples, the CUDA negative fixture, workflows, scripts, manifests and documentation/evidence. Independent baseline and architecture reviewers plus a focused dependency reviewer supported parent validation. New remediation received source review and the checks below. Only the nine locally available Git commits/refs were secret-scanned; no claim covers deleted/unavailable remote history.

Inspected the all-feature Cargo graph (135 packages including Gudra; 134 checksummed registry dependencies), every listed build-script target source (41), every proc-macro entry (10), and material native/process/filesystem helpers. No Git/path dependency or first-party build script exists. Native-source/prebuilt/linking inventory marks 16 packages; some files are test fixtures or target-conditional, not automatically executed on Linux.

Reviewed relevant CUDA/cuTile unsafe allocation, launch, stream, readback, cancellation, cache and compiler paths. The [unsafe inventory/review](unsafe-boundaries.md) explicitly separates enumeration from complete implementation review. There are zero handwritten first-party Rust unsafe constructs. The dependency lexical inventory contains 27,288 unsafe-token locations across source, tests, comments and generated templates; those are **not** 27,288 independently audited blocks.

Ignored build outputs, unrelated sibling projects, proprietary CUDA/compiler internals, expanded generated machine code, live GPU fault injection, external repository settings and remote release/owner configuration are not certified. Historical GPU logs were read as supplied evidence, not rerun. The plugin snapshot included ignored build outputs and reported an unversioned target due to repository ownership detection; this report's source revision and scope are from explicit local Git inspection. Coverage is therefore **partial for the full requested backend/release gate**, despite complete first-party baseline review.

## Findings and dispositions

| ID | Severity/status | Result and release treatment |
|---|---|---|
| [SEC-001](findings/SEC-001-jit-temporary-files.md) | Medium, validated isolation gap; execution impact conditional | Transient .bc/.cubin pathname exchange lacks private per-compilation directory. Private Linux worker launcher added; direct library callers still need deployment containment/upstream fix. Owner: runtime/dependency maintainer; due 2026-09-26. Not accepted for general release |
| [SEC-002](findings/SEC-002-resource-admission.md) | Informational hardening | Added configurable logical allocation/buffer/operation budgets and static request preflight. Conditional service DoS, not an unconditional library vulnerability. Final GPU wiring test pending |
| [SEC-003](findings/SEC-003-delivery-hardening.md) | Informational hardening | CI least privilege/pinning/locked checks and dependency/secret policy automated. External protections remain unverified |
| [SEC-004](findings/SEC-004-failed-drain-proof-gap.md) | Informational safety proof gap; no exploit established | Failed drain leaks a borrowed child tuple, not outer owners. Normal cancellation/readback evidence does not establish fault safety. Blocks the strict unsafe-boundary acceptance criterion |

All records use [templates/SECURITY_FINDING.md](../../templates/SECURITY_FINDING.md). Full scoped local issue specifications, proposed accountable owners, deadlines and closure criteria are in [follow-up-issues.md](follow-up-issues.md). No issues were posted externally.

## Memory, execution and abuse checks

Checked geometry covers zero axes, each arithmetic overflow stage, isize byte bounds, i32 counts/strides, total elements and each 16-cell tile-grid axis. Exact host lengths are checked before writes/uploads. Fixed internal view ranges retain halo row stride; public metadata and storage are private. No safe alternate mutable view, raw pointer, unchecked indexing, foreign-stream import or cross-session bypass was found.

The new ResourceLimits defaults are 256 MiB/buffer, 1 GiB/session, 64 buffers and 8 admitted operations. Admissions are atomic, reject overflow and roll back on ordinary error/unwind/drop. An unpolled future retains its operation and buffer charges; forgotten owners retain charges. Static checks reject a valid 4 GiB shape without allocating it. Host inputs/readback vectors, allocator caches and delayed frees, JIT/compiler overhead, multiple sessions, request rate and deadlines require application/OS limits. This is logical admission, not physical-memory isolation.

Ordinary submitted cancellation waits on the stream and may block. Borrowed execution invalidates output on error/unwind; owned failures return no partial owner. Readback retains an independent Arc through sync. Failed drains and context recovery remain unproved. Forced-pending cancellation, error-path physical reclamation and final new GPU budget wiring were not experimentally established.

No arbitrary device-code/data-to-source API is exposed. Pinned JIT key analysis contradicts one-variant-per-dimension/float-value exhaustion: extents absent, divisibility capped at 16, float scalars not value-specialized, grid not constant-specialized. Persistent caching defaults off; enabled default store has only a soft 2 GiB eviction target and requires trusted writers. No unlimited application retry loop exists. Worker memory/time/disk quotas remain deployment responsibilities; destructive huge-allocation/JIT stress was deliberately replaced by bounded admission tests and source analysis.

## Supply chain and CI

`Cargo.lock` SHA-256: `d37f7e457de840c2a010a79ba85ee3a7d5ea096569937fab40cc877b6c387a1c`. Direct cuTile remains exactly 0.3.1; the lockfile was unchanged. The [capability inventory](dependency-inventory.json) and its checker reject changed identities/versions/checksum references/licenses/build/proc-macro/native capabilities and unexpected Git/path/registry sources. They are not a source-cache attestation or sandbox.

Build/macro review found compiler feature probes and OUT_DIR generation; bindgen/libclang and CUDA headers; psm assembly/prebuilt wasm object; stacker Windows C; packaged Windows import archives; rustc/wrapper, llvm-config, platform probes and git commands. No direct downloader/network client was found in inspected build/macro paths. Executable build code still inherits the build user's authority. Environment/PATH/native-loader selection and external toolkit binaries must be trusted.

[Cargo-deny](https://embarkstudios.github.io/cargo-deny/) 0.20.2 performed RustSec advisory, license, bans and source checks with all features. Database revision `d5c17953a895cf19e8d3ce66eaa42b6fcfe1fb16`, dated 2026-09-19, was fetched before the audit and reused for captured offline evidence. All four checks passed; duplicate versions and one unused license allowance were warnings, not advisory failures. There are no ignored advisories. Unpublished Gudra itself is excluded from license checking because no owner-selected license exists; publication remains disabled. Rust license results do not establish CUDA redistribution rights.

[Gitleaks](https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1) 8.30.1 found zero current-source/history findings, with explicit default rules, no inline allow bypass and 100% redaction. This is a pattern-based scan, not proof no secret ever existed. Raw matched values and scanner diagnostics are not published.

Security tools were downloaded from official releases and checked against published SHA-256 values before execution; CI embeds those reviewed digests. Checkout v4.3.1 resolves to `34e114876b0b11c390a56381ad16ebd13914f8d5`, verified from the publisher's GitHub tag API. CI declares contents:read, persist-credentials:false, hosted Ubuntu 24.04, 20-minute timeouts, locked builds and weekly security/dependency checks. No release secrets, sensitive GPU runner, executable artifact imports or artifact uploads were added. Required-check enforcement, private reporting, 2FA/ownership and 7-day log retention need repository-owner verification.

## Verification

| Check | Outcome/evidence |
|---|---|
| CPU debug and release | Each 23 tests plus 1 doctest pass; [debug](evidence/cpu-debug.txt), [release](evidence/cpu-release.txt) |
| CPU ownership UI | Positive control and E0502 alias rejection pass; [output](evidence/cpu-ui.txt) |
| Rustfmt / CPU Clippy / docs | Pass; Rust 1.90 formatter/lint tools, Rust 1.89 test compiler/rustdoc. [docs](evidence/cpu-docs.txt) |
| Resource admission | CPU overflow/large-shape/concurrent/count/operation/unwind/forget tests pass; no huge allocation made |
| Dependency policy | 134 registry dependencies, 41 build scripts, 10 macro entries; snapshot check pass; [output](evidence/dependency-policy.txt) |
| RustSec/license/bans/sources | All pass, known nonblocking duplicate/unused-allowance warnings; [output](evidence/dependency-check.txt) |
| Secrets | Current source and 9 available baseline Git commits clean; final scan evidence saved separately |
| Python scripts | Syntax checked; installer/secrets/inventory/private-temp source reviewed |
| New GPU compile | **Blocked**, cuda-bindings missing CUDA toolkit, before changed facade type-check; [output](evidence/gpu-build.txt) |
| GPU UI/runtime/sanitizers/forced-pending/fault injection | Not executed in this audit; prior 073f7c6 results retained as historical evidence only |
| Linux private-temp launcher and hosted CI | Source reviewed; Linux/remote execution pending |

Local workaround uses installed Cargo 1.90 with explicit Rust 1.89 compiler/rustdoc paths because the pinned local toolchain lacks Cargo. A first offline metadata attempt lacked target-specific cached packages; exact locked packages were fetched without executing them. The first captured offline cargo-deny attempt could not lock a read-only sandbox cache; repeating with authorized cache access passed. Neither environmental failure was treated as a vulnerability or a passing check.

## Acceptance decision

| Required criterion | Disposition |
|---|---|
| No unresolved critical/high | Met within reviewed evidence; none validated |
| Medium owner/deadline/rationale | Recorded for SEC-001; owner assignment needs confirmation; no blanket risk acceptance |
| Every unsafe boundary documented and tested or removed | **Not met**: generated/runtime fault paths and final GPU changes unverified |
| Automated dependency and CI policies | Implemented locally; hosted run/enforcement pending |
| Public claims match evidence | README/API/setup and SECURITY updated; no production isolation/fault-recovery claim |
| Supported reporting/release provenance/ownership | Policy specified; settings/license/first-release proof pending |

**FAIL remains the release decision.** Reopen only after the scoped follow-ups close with evidence tied to the final immutable revision. No publication, CVE request, upstream disclosure, credential changes or destructive external tests occurred. The [handoff](../handoffs/06-security-threat-model-and-release-gate.md) identifies the next work.

`python3 scripts/check-release-gate.py` enforces the recorded failure and checks
the source/configuration digest in [release-gate.json](release-gate.json). It
currently exits 1 as intended. This local guard must be wired into any future
publisher; it does not claim enforcement by an absent release workflow. No local
PTX/cubin/Tile IR bytecode or dumped launcher artifacts were present under target
for binary inspection; generated device artifacts still need GPU validation.
