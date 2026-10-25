# Gudra 0.1.0-rc.1 — source-only preview (GPU UNVERIFIED)

Gudra is an experimental source distribution for alias-safe weighted Jacobi and
Laplacian stencils, with a scalar CPU reference and optional cuTile GPU backend.
The name was approved by the owner on 2026-10-12. Version `0.1.0-rc.1` is the
local release candidate for a source-only GitHub channel, retaining all reviewed
backend patches. The owner subsequently authorized this source-only prerelease
on 2026-10-25; crates.io publication remains disabled.

**GPU UNVERIFIED.** This prerelease is for source evaluation and CPU reference
use. No GPU execution, fault-injection or sanitizer pass is claimed for this final
revision. Current security reviews report zero findings; pinned Rust 1.89 CPU,
ownership compile-fail, doctest, docs, package and security checks pass. Protected
final CI and signed artifact provenance must verify before publication. Review the
[supported configurations](supported-configurations.md) and
[preview authorization](preview-authorization-2026-10-25.json).

The default feature set builds without CUDA. The GPU source build must retain
Cargo.lock, all three vendor overrides, their integrity manifest and Apache-2.0
licenses. The Cargo archive is a packaging experiment: Cargo removes root patch
overrides and resolves registry versions. Its GPU backend lacks our fixes;
crates.io publication is blocked even if a default-feature package build passes.
No Git dependencies are accepted by the source policy. All other locked Rust
dependencies are registry sources; see the dependency-policy evidence.

One-cell halos, row-major `f32`, caller-supplied boundaries and fixed-Dirichlet
timestep batches are the supported model. No convergence, fresh-halo, universal
floating-point error bound, multi-GPU correctness, driver recovery, tenant
isolation or overlap-performance guarantee is made. GPU faults may abort the
worker; replace it rather than retrying in that process. Cancellation may block.
Timing evidence excludes transfers/JIT and uses a scalar CPU comparison.

The source candidate intentionally excludes benchmark dumps, historical evidence
directories, generated binaries/PTX/cubins, caches, local tooling and the standalone
halo prototypes. Source documentation can reference excluded evidence; use the
reviewed repository for those records. The public repository currently exposes an
older candidate, so local current-source documentation must not be advertised as
already accessible there.

Gudra is MIT; modified backend crates remain Apache-2.0 with notices retained.
CUDA toolkit, driver and tileiras are external prerequisites and are not bundled.
Report vulnerabilities through SECURITY.md before public disclosure.
