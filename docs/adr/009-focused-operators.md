# ADR 009: one explicit scalar operator, no new policy or dimensionality framework

Status: accepted; v2 runtime validation passed on the recorded Arch experimental
configuration (see Chat 09 handoff).
Date: 2026-10-03.

The user requested growth only where the MVP proves shared abstractions. Chat 08
has numerical and sanitizer evidence for 2D f32 and private output owners on an
RTX 5060 Laptop GPU. Its 17x19 scalar-tile batch timing with spinning disabled
shows owned async is slower than blocking; it does not justify a new scheduler,
3D layout, f64, or vector/fused operators.

Select only the five-point Laplacian, the first candidate. It is directly useful
for measuring the residual of the existing Poisson/Jacobi operation and checking
curvature. These examples demonstrate local use, not external customer demand.
Heat, gradient/divergence, reaction-diffusion, 3D and f64 remain deferred until
application evidence and configuration-specific timings exist.

Keep a concrete LaplacianParams with isotropic positive squared spacing. Cache a
finite reciprocal and preserve the documented arithmetic order. Do not alter
Jacobi's permissive zero spacing domain. Nonfinite fields propagate. No generic
precision trait, boundary policy, dimensionality trait, new macro or DSL is added.
The existing cuTile entry macro remains the backend mechanism.

The Laplacian reuses Shape2D, HaloGrid2D, Field2D, directional_views, PartitionMut,
Budget and Completion. Extract validate_fields from validate_step to validate
an optional RHS with the same checks; Jacobi delegates to it unchanged. Keep
explicit device arithmetic beside Jacobi rather than introduce generic operator
callbacks. This preserves compiler diagnostics and makes generated code review
straightforward. All source code retains unsafe_code=forbid.

Boundary support is caller-supplied one-cell samples only; automatic policies,
anisotropic spacing, padded/pitched/transposed layouts and 3D have no public API.
Wrong slice lengths, shapes, sessions and private metadata fail before writes;
f64 slices and a halo owner used as output are rejected by Rust types. The API
cannot detect a stale or scientifically wrong halo supplied with the right size.

Cost: a few repeated load expressions and a separate scalar oracle. Reconsider
abstraction after at least three measured operators show the same recurring
structure. A reduction for residual/convergence remains a separate host operation
and carries no partition/disjoint-write safety claim.
