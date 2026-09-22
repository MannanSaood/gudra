# Security policy

Gudra is experimental, unpublished (`0.0.0`, `publish = false`), and **not approved for production or mutually untrusted GPU tenants**. The current [release gate](docs/security/audit-report.md) is **FAIL**. This is a release-readiness decision, not a claim that a critical/high exploit was found.

## Supported versions

| Version | Security support |
|---|---|
| Current reviewed main development revision | Best-effort investigation; fixes land on main |
| Published releases | None |
| Historical revisions, forks, arbitrary dependency versions | No backport commitment |

## Private reporting

Use the repository's [GitHub private vulnerability reporting page](https://github.com/MannanSaood/gudra/security/advisories/new). **The maintainer must enable and test this channel before release; availability was not verified in the local audit.** If the page is unavailable, ask the maintainer for a private reporting channel without including vulnerability details in a public issue. Do not submit credentials, private grids or customer data.

Include affected revision, dependency/toolchain versions, deployment assumptions, a minimal safe reproducer, expected/actual behavior and impact. Proposed response targets are acknowledgment within 3 business days, initial triage within 7 days, and progress updates at least every 14 days; these are best-effort targets, not an emergency response SLA. The maintainer owns triage and coordination. No CVE request, upstream disclosure or public vulnerability publication is authorized by this audit.

## Scope and guarantees

Security scope includes cross-boundary host/device memory access, numerical-result corruption affecting another principal, resource exhaustion in a service accepting untrusted requests, dependency/JIT artifact integrity and CI/release authority. Ordinary local misuse, stale halos, unstable coefficients, NaN propagation or lack of convergence are not automatically vulnerabilities.

Safe APIs keep device storage private, validate geometry and session identity, and exclude public mutable aliases. This does not certify cuTile/CUDA/compiler/driver internals, shared-GPU isolation or fault recovery. Unsuccessful synchronization is not proof that storage is safe to release or reuse. Discard the worker process/session after device faults; do not retry within it.

## Deployment requirements

Use trusted source, locked dependencies, compiler installations and native libraries. Build without production credentials. Run mutually untrusted jobs under separate OS identities and an operator-validated GPU isolation arrangement; a `Gpu` session or container alone is not a certified tenant boundary.

The local compiler override now creates private per-compilation Linux storage under validated root-owned `/tmp`, including bytecode-version probes. Linux helper unit/separate-UID tests passed in hosted CI at 401b665; actual compiler integration and GPU execution remain pending, so SEC-001 is not closed. The override does not certify persistent caches or same-UID isolation. Start GPU workers with additional private temporary storage using `python3 scripts/run-private-gpu.py -- <trusted command>`, or equivalent verified ownership/permissions configured before process startup. The launcher is defense in depth; the compiler override also covers direct library calls. Keep persistent cuTile caching disabled unless its storage and writers are trusted and disk quotas enforced.

Apply ingress/file-size limits before reading data or allocating host vectors. Use `ResourceLimits::check_shape` and `Gpu::with_limits`; also bound process memory, number of sessions, request rate, runtime and retries. Budgets count logical storage, not allocator caches, physical free completion, compiler memory or host vectors. Do not return backend diagnostic strings or raw GPU logs to untrusted clients.

See [deployment and release follow-ups](docs/security/follow-up-issues.md), the [threat model](docs/security/threat-model.md) and [unsafe-boundary review](docs/security/unsafe-boundaries.md).

The owner authorized agent selection of a project license on 2026-09-22; Gudra is now MIT licensed. The vendored compiler remains Apache-2.0 with its notices retained. This does not authorize redistribution of proprietary CUDA components.
