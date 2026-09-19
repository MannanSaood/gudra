# Security Finding

## Metadata

- ID: SEC-001
- Title: cuTile transient compiler output lacks a private temporary-directory boundary
- Status: validated (isolation gap); deployment mitigation added, upstream closure pending
- Severity: medium
- Affected versions/components: locked cutile-compiler 0.3.1, reached through Gudra gpu feature
- Reporter/date: independent local audit, 2026-09-19
- Owner/deadline: runtime/dependency maintainer, 2026-09-26
- Confidence: high in source facts; medium in conditional interference; executable substitution untested
- CWE: CWE-377

## Summary

The runtime compiler writes bytecode and a related cubin filename into the process temporary directory without a private per-compilation directory. A co-user who can observe entries may interfere with the output name. Executable substitution depends on tileiras file-open behavior and OS protections; this is not a claim of universally exploitable GPU or host code execution.

## Threat model

- Attacker capability: list/create entries in shared temp, or write access from an overly broad ACL.
- Required access: local co-user distinct from the victim worker; first JIT compilation.
- Trust boundary crossed: another filesystem principal influences bytes consumed as victim device code.
- Assets affected: victim result/device-memory integrity and availability; no demonstrated host execution.

## Evidence and reproduction

Registry source, checksummed by Cargo.lock: `cutile-compiler-0.3.1/src/cuda_tile_runtime_utils.rs:872-878` creates UUID-based `.bc` and `.cubin` paths beneath `env::temp_dir()`, then uses `std::fs::write` for bytecode. Lines 894-895 pass `-o <same UUID>.cubin` to tileiras. Line 925 reads the output by pathname. Lines 931-938 require only a nonempty successful output. `ScopedTempFile:696-719` is path cleanup, not exclusive reservation or a private directory. `cutile-0.3.1/src/tile_kernel.rs:806-821` forwards bytes into an unsafe module loader.

Safe validation was source tracing only: bytecode publication makes the related output name observable before compilation; a precreated output can belong to the co-user. No malicious binary was compiled/loaded and no cross-user test was run. UUID unpredictability, normal permissions, sticky directories, Linux protected_regular/protected_symlinks and compiler exclusive/replace behavior are meaningful counterevidence. They may reduce the outcome to a failed compilation. Windows local ACL observations are not evidence about the supported Linux GPU environment.

## Impact

Compiler-output interference can deny a victim compilation. If substituted valid cubin bytes are accepted, code executes in the victim GPU context and can corrupt memory/results or consume GPU resources. Malformed bytes reach the driver, but driver compromise is unproven. Medium reflects local access, timing and unresolved compiler/OS prerequisites; no high finding is claimed.

## Root cause

The runtime trusts a pathname whose name becomes known before output creation, without maintaining a private writer boundary throughout compiler execution and readback. Persistent-cache settings do not disable this transient path.

## Remediation

- Immediate containment: launch the whole worker through `scripts/run-private-gpu.py`; `verify-gpu.sh` now does so automatically. It validates Linux parent ownership/write/sticky policy, atomically creates mode-0700 temp storage and sets child TMPDIR/TMP/TEMP before startup.
- Code fix: upstream/private reviewed compiler patch must create a private per-compilation directory and preserve it through output reading and all error paths. No dependency patch was silently vendored.
- Regression test: distinct-UID creation/read/replacement attempts against private files, compiler output/open behavior, failure cleanup and path attacks on an isolated Linux test host.
- Documentation: shared-service accounts/caches are not isolation; direct library callers must establish the private-temp condition.

## Verification

Private launcher source and syntax reviewed; Linux execution and hostile-principal regression remain pending. Existing CPU checks do not test this boundary. See GATE-LINUX and SEC-001 in follow-up-issues.md. Release rationale: containment is available for one controlled launch path; general release remains blocked until closure evidence.

## Disclosure

Local report only. No issue, upstream disclosure, vulnerability publication or CVE requested. Coordinate privately only with explicit user authorization.
