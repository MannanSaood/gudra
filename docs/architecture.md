# Architecture

The public CUDA-free layer checks Shape2D extents, element/byte limits,
Jacobi/Laplacian coefficients, and resource admission. The scalar reference uses
bounds-checked indexing. With `gpu`, Gpu creates a private ordered session;
HaloGrid2D is an immutable source and Field2D is an interior RHS/output role.
Neither exposes a backend allocation, stream, pointer or clone.

Single-step kernels read five shifted directional views of one immutable haloed
source, each with the interior extent. A 16x16 output partition writes compact
interior storage, including masked partial tiles. The aliases among read-only
views are intentional; output must be a separate allocation. Directional views
are internal implementation details, not a public user-configurable layout API.

DoubleGrid privately owns two haloed allocations and RHS. Each fixed-Dirichlet
step reads one allocation, writes the other and swaps roles after completion.
Its scalar-tile kernel copies the boundary. Consuming advance/advance_async
prevents access to pending state; observe returns ready state plus a host snapshot.

The backend compiles/JITs and submits CUDA work. Completion guards invalidate
uncertain borrowed output; patched terminals drain submitted work. If completion
cannot be proven they abort the worker before freeing owners. Backend unsafe code,
JIT/native libraries, driver and hardware remain trusted obligations.

The standalone multi-GPU prototype adds epoch/generation checked slab exchange,
P2P or pinned-host staging and supervisor cleanup. It is separate from the public
crate and awaits real two-GPU evidence.

See [safety](safety-model.md), [unsafe boundaries](security/unsafe-boundaries.md),
[owned async ADR](adr/008a-owned-async.md), [double-buffer ADR](adr/008b-double-buffer.md),
[graph deferral](adr/008c-graphs.md), [focused operators](adr/009-focused-operators.md),
and [roadmap](roadmap.md).
