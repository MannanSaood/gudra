# Security policy

Gudra is experimental and unpublished (`0.1.0-rc.1`, `publish = false`). The
historically reviewed candidate passes its bound technical release gate. The
current source-only candidate completed a fresh static diff review on
2026-10-25 with no reportable findings; see
[current release status](docs/release/status-2026-10-25.md). The full release
gate remains on HOLD for the remaining execution and provenance checks.
The historical decision is not approval
for production or mutually untrusted GPU tenants and does not authorize a tag,
GitHub release, or package publication. No critical or high vulnerability was
confirmed.

## Supported versions

| Version | Security support |
|---|---|
| Current reviewed main development revision | Best-effort investigation; fixes land on main |
| Published releases | None |
| Historical revisions, forks, arbitrary dependency versions | No backport commitment |

## Private reporting

Use the repository's [GitHub private vulnerability reporting page](https://github.com/MannanSaood/gudra/security/advisories/new). Private vulnerability reporting was verified enabled on 2026-09-26. If the page is unavailable, ask the maintainer for a private reporting channel without including vulnerability details in a public issue. Do not submit credentials, private grids or customer data.

Include affected revision, dependency/toolchain versions, deployment assumptions, a minimal safe reproducer, expected/actual behavior and impact. Proposed response targets are acknowledgment within 3 business days, initial triage within 7 days, and progress updates at least every 14 days; these are best-effort targets, not an emergency response SLA. The maintainer owns triage and coordination. No CVE request, upstream disclosure or public vulnerability publication is authorized by this audit.

## Scope and guarantees

Security scope includes cross-boundary host/device memory access, numerical-result corruption affecting another principal, resource exhaustion in a service accepting untrusted requests, dependency/JIT artifact integrity and CI/release authority. Ordinary local misuse, stale halos, unstable coefficients, NaN propagation or lack of convergence are not automatically vulnerabilities.

Safe APIs keep device storage private, validate geometry and session identity, and exclude public mutable aliases. This does not certify cuTile/CUDA/compiler/driver internals, shared-GPU isolation or fault recovery. Unsuccessful synchronization is not proof that storage is safe to release or reuse. Discard the worker process/session after device faults; do not retry within it.

## Deployment requirements

Use trusted source, locked dependencies, compiler installations and native libraries. Build without production credentials. Run mutually untrusted jobs under separate OS identities and an operator-validated GPU isolation arrangement; a `Gpu` session or container alone is not a certified tenant boundary.

The compiler override creates private per-compilation Linux storage under
validated root-owned `/tmp`, including bytecode-version probes. Hosted Linux
helper and separate-UID tests passed, and the actual compiler/JIT-to-GPU-load
lifecycle passed on the isolated GPU worker. SEC-001 is closed for supported
invocation paths. The override does not certify persistent caches or same-UID
isolation. Start GPU workers with additional private temporary storage using
`python3 scripts/run-private-gpu.py -- <trusted command>`, or equivalent verified
ownership/permissions configured before process startup. The launcher is defense
in depth; the compiler override also covers direct library calls. Keep persistent
cuTile caching disabled unless its storage and writers are trusted and disk
quotas enforced.

Apply ingress/file-size limits before reading data or allocating host vectors. Use `ResourceLimits::check_shape` and `Gpu::with_limits`; also bound process memory, number of sessions, request rate, runtime and retries. Budgets count logical storage, not allocator caches, physical free completion, compiler memory or host vectors. Do not return backend diagnostic strings or raw GPU logs to untrusted clients.

See [deployment and release follow-ups](docs/security/follow-up-issues.md), the [threat model](docs/security/threat-model.md) and [unsafe-boundary review](docs/security/unsafe-boundaries.md).

The owner authorized agent selection of a project license on 2026-09-22; Gudra is now MIT licensed. The vendored compiler remains Apache-2.0 with its notices retained. This does not authorize redistribution of proprietary CUDA components.

## Release governance

The repository owner selected solo-maintainer governance on 2026-09-26. While
there is one maintainer, an independent pull-request approval is not required;
this must not be represented as independent review. Protected `main` still
requires the current CPU and security checks with administrator enforcement,
linear history and resolved conversations. Force pushes and deletion are
blocked, and release tags matching `v*` are immutable. GitHub Actions are
restricted to SHA-pinned GitHub-owned actions, read-only tokens by default and
a protected release environment. MFA, private reporting and minimal ownership
must remain enabled. Adding another release-review maintainer requires an
approving review before the next release.

Release evidence must come from an approved immutable `main` revision on a
clean isolated worker and include the dependency/toolchain identities, SBOM,
SHA-256 checksums and signed provenance described in
[the release process](docs/release-process.md). A passing gate is a technical
readiness decision only; publication remains a separate explicit owner action.
