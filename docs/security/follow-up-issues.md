# Scoped release follow-ups

## Final public candidate update — 2026-09-25

Public candidate `1ee62e582912590ae27af6256b6da9c6262aed1c` is live on
`main`; hosted CPU/security run 36125581179 passed. SEC-004 has a reviewed
fail-stop/quarantine implementation for Gudra's safe GPU facade, including
blocking, async, cancellation, upload, readback, and raw-allocation error paths.
It remains pending until this exact revision passes the supplied real-device
state, fault, numerical, cancellation, and sanitizer tests. SEC-001 remains
pending for the same final-revision compiler/JIT execution. No historical GPU
result is attributed to this candidate.

The release decision remains **FAIL**. Private reporting, branch/release-ref
protection, required reviews/check enforcement, maintainer MFA/minimal package
owners, retention, and signed provenance remain external owner checks. These
private audit records were not included in the public commits.

## Chat 07 candidate update — 2026-09-22

This update supersedes older statements above about the unchanged registry compiler
and absence of a project license. Release remains **FAIL**. SEC-001 has a local
Apache-2.0 compiler override: both run_tileiras and version probing use private
0700 directories under root-owned /tmp, exclusive bytecode creation, regular-file
output checks and cleanup on return/unwind. Root, compiler and same-UID processes
remain trusted. SIGKILL/abort can leave private directories; operator cleanup is
required only after child exit. No process-global environment mutation was added.
Linux helper unit/separate-UID tests passed in hosted CI at 401b665. Actual compiler integration and real GPU execution are pending;
this is implemented mitigation, not verified closure.

SEC-004 remains unresolved. Independent review traced synchronous sync_on dropping
results on failed synchronization, execute error/unwind before a result is retained,
and borrowed async result leaks that fail to retain outer allocations/session.
No partial outer-owner guard was represented as a solution. No destructive device
fault was injected. No critical/high exploit was established by this review.

Current CPU debug/release, UI, docs, formatting, Clippy, installer tamper controls,
locked dependency inventory, advisory/license/source/bans and secret checks passed
locally. GPU build exits 101 before facade/compiler verification due to missing
CUDA. Hosted CPU/security CI, Linux launcher tests, verified installer and separate-UID helper tests passed at public candidate 401b665 (run 35736906616). Repository protections, private-reporting functionality and signed provenance remain unverified. MIT was chosen under explicit
owner authorization. See the private Chat 07 handoff for candidate identities,
publication separation, exact remaining commands and acceptance criteria.

## Historical review record

The following records prior observations. Current changes and limitations are above.

These are local issue specifications, not published GitHub issues. Owners below are accountable roles proposed for maintainer assignment; acceptance of the assignments is unverified. Due dates are 2026-09-26 unless stated otherwise. Nothing here authorizes public disclosure or upstream vulnerability submission.

| ID | Owner | Deadline | Required closure evidence | Release rationale |
|---|---|---|---|---|
| SEC-001 | Runtime/dependency maintainer | 2026-09-26 | Patch or reviewed upstream release uses private per-compilation directory; Linux test proves other UID cannot create/read/replace artifacts; verify compiler success/failure cleanup and symlink handling | Medium; direct library launch still vulnerable to transient-path interference under stated prerequisites. Private-temp launcher is containment for one deployment path, not blanket acceptance |
| SEC-004 | Runtime safety maintainer | 2026-09-26 | Document all post-submission errors/panics/drop paths; owned retention until successful drain or permanent quarantine; isolated fault tests demonstrate no deallocation before last use | Safety proof gap, not confirmed exploit. Release fails until evidence or API/design restriction closes it |
| GATE-GPU | GPU validation maintainer | 2026-09-26 | Exact final commit, full GPU compile/clippy/UI/doctests, numerical/probe/readback/budget tests, successful max-operations=1 async completion, forced-pending cancellation, memcheck/initcheck with stream-order tracking | CPU tests and historical 073f7c6 results cannot certify new GPU wiring |
| GATE-LINUX | CI maintainer | 2026-09-26 | Hosted CPU/security workflow green; Linux private launcher owner/mode/ancestor rejection and cleanup tests; native tool installer checksum validation exercised | Local Windows syntax/review is not Linux runtime evidence |
| GATE-RELEASE | Repository owner | Before first release, no later than 2026-09-26 review | Enable/test private reporting; require reviews and CPU/security checks; protect main/tags/workflows; verify maintainer 2FA and minimal crate owners; choose license | External settings and publishing rights cannot be proved from local files |
| GATE-PROVENANCE | Release maintainer | Before first release, no later than 2026-09-26 review | Build from reviewed immutable tag in clean isolated worker; record lockfile/toolchain/native-tool digests and SBOM; SHA-256 every package/native artifact; bind signed provenance/attestation to commit and artifact digests | No release pipeline exists. Checksums alone do not authenticate origin; never promote a PR-produced binary/cubin |
| GATE-OPERATIONS | Deployment owner | Before any untrusted-request/shared-GPU pilot | Input/file/process memory quotas, bounded session count/request rate/deadlines/retries, private UID/temp/cache, GPU isolation assessment and fault-worker replacement | Session budgets are logical admission, not hardware isolation or total physical-memory limits |

## SEC-001 implementation scope

Change `cutile-compiler 0.3.1 run_tileiras` to own a private temporary directory for the complete bytecode/compiler/output-read lifecycle. Use atomic directory creation with mode 0700 or restricted Windows ACL, no untrusted parent replacement, and regular-file/no-follow handling where appropriate. Preserve diagnostic utility without leaving executable intermediates in a shared directory. Review tileiras file-open behavior and test on a disposable GPU host; do not test malicious cubins on a sensitive host. Pin the resulting reviewed package/revision and re-run dependency policy.

Current containment is `scripts/run-private-gpu.py`, automatically entered by `scripts/verify-gpu.sh`. Direct consumers must invoke their entire process through an equivalent private-temp boundary before any CUDA/JIT use. No process-global environment mutation was added to the multithreaded Rust library.

## SEC-004 implementation scope

The child DeviceFuture leaks its result on failed drain, but that result contains borrowed arguments; it does not retain outer owners. Audit synchronous error/unwind terminals too. Options include an owned backend operation that retains every allocation until confirmed completion, permanent quarantine on uncertainty, or a separate blocking worker owning all allocations whose process is discarded on device fault. Avoid merely leaking reference tuples or marking numerical output invalid. Add deterministic fault seams at submission/completion and real isolated fault evidence without manufacturing a successful test from unavailable CUDA.

## Release pipeline and credentials

Keep `publish = false` until the gate closes. No workflow should combine untrusted PR code with publication credentials or sensitive GPU access. If a release job is later added, use a protected environment with maintainer approval and short-lived narrowly scoped identity; its input must be the approved immutable revision, not downloaded PR build artifacts. Review hashes/signatures/provenance before loading JIT/native images or publishing packages.

Set hosted CI logs to at most 7 days in repository settings and confirm access; this setting is external and unverified. The workflow uploads no artifacts. GPU logs remain operator-private with a 7-day working retention target; reviewed redacted summaries tied to a revision may be retained as release evidence. Never upload raw compiler dumps, host paths/topology, data arrays, secrets or imported executable artifacts by default. Existing evidence contains non-secret host/path details; review its intended public audience before publishing the repository.

License policy applies to resolved Rust dependencies. It does not grant a Gudra license or establish CUDA toolkit redistribution rights. The repository owner must select the project's license and verify native redistribution terms before publication.
