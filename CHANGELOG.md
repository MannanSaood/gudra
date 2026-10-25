# Changelog

## Unreleased source candidate — 2026-10-12

Release name: Gudra (`gudra`). Candidate version: `0.1.0-rc.1`, selected for the
source-only GitHub channel under the owner's request to resolve release blockers.
Publication remains disabled; no public release tag exists for this snapshot.

- Weighted five-point Jacobi and five-point Laplacian CPU/GPU APIs on haloed
  row-major `f32` grids, with explicit source/output roles and RHS borrowing.
- Fixed-Dirichlet timestep batches retain double-buffer ownership through
  completion and cancellation; fail-stop behavior protects uncertain completion.
- Checked shapes, parameters and resource admission; numerical tests, ownership
  compile-fail controls, cancellation seams and readback tests.
- Reviewed local Apache-2.0 backend patches for private JIT storage and completion
  retention; these are required for the supported source build.
- Runnable CPU examples, numerical conventions, setup/troubleshooting guides,
  scoped benchmark evidence and contribution/reporting guidance.
- Release metadata, package allowlist, archive inspection, fail-closed CI gate,
  provenance, support, rollback and security-response procedures.

The standalone halo prototype is excluded from release artifacts and is not a
public multi-GPU API. Current GPU validation and security approval remain open;
historical passes cover their recorded sources only. See
[release notes](docs/release/notes.md) and [checklist](docs/release/checklist.md).
