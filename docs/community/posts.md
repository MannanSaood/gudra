# Community post drafts

Prepared 2026-10-12; none posted. These are maintainer-voice drafts, to be edited
by the sender. Do not imply NVIDIA affiliation or endorsement. Before posting,
make the documented candidate accessible and replace revision references with a
public source permalink. Confirm the conduct-policy enforcement owner. Read each
venue's current rules, search for duplicate discussions and choose one relevant
channel/topic. Post once per venue, respond to actual questions, and do not bump
for visibility or ask for stars. Stagger venues so early feedback can improve the
next draft. Stop reposting if moderators decline it.

Destinations checked on 2026-10-12:
[cuTile Discussions](https://github.com/NVlabs/cutile-rs/discussions) exposes General,
Ideas and Q&A; choose General or Ideas for design feedback.
[Zulip's community directory](https://zulip.com/communities/) links
[Rust-SciComp](https://rust-scicomp.zulipchat.com/); authenticated stream/topic
names were not inspected, so choose a relevant scientific-computing stream after
joining. [r/rust rules](https://www.reddit.com/r/rust/about/rules/) were not readable
in the fetched page; the sender must read them in the UI before submission.
The fourth draft targets [NVIDIA CUDA Programming and Performance](https://forums.developer.nvidia.com/c/accelerated-computing/cuda/cuda-programming-and-performance/7),
rather than duplicating it on r/CUDA. Category availability does not imply moderator approval.

## NVlabs cuTile Rust Discussions

Title: Design feedback: separate halo/input ownership for five-point stencils

I'm working on Gudra, an experimental Rust wrapper using cuTile 0.3.1 for
weighted Jacobi and a five-point Laplacian. The failure I want to exclude is a
Jacobi update overwriting storage that neighboring cells still read as the old
iteration. HaloGrid2D and Field2D keep source/output roles separate; owned async
calls move buffers until completion. This is an API ownership boundary, with
cuTile/CUDA still trusted, not a proof of all GPU race freedom.

The current layout is compact row-major 2D f32 with a one-cell halo. Internally,
five shifted read-only views have the interior extent and feed a 16x16 output
partition. Fixed-Dirichlet batches own two haloed allocations. Numerical/UI and
sanitizer evidence is revision-scoped; the separate two-GPU prototype has no
hardware acceptance yet.

- Do these internal directional views map to the halo storage you actually use, or do pitch, subdomains, or packed faces require a different interface?
- Would consuming buffer owners until an async call completes prevent a bug in your code, or conflict with your executor/stream model?
- Which minimal view/partition tests would you want before trusting a wrapper like this?

[Project and evidence](https://github.com/MannanSaood/gudra). A small synthetic
layout or workflow counterexample would help more than a general endorsement.

## Rust-SciComp Zulip

Topic: Halo storage and boundary needs for an ownership-based stencil API

I'm developing Gudra, a small experimental Rust/cuTile stencil library, and I'd
like feedback from people who maintain scientific solvers. It keeps an old haloed
grid separate from mutable output so Jacobi cannot accidentally become an
in-place mixed-iteration sweep through the safe public API. There is a CUDA-free
scalar reference and minimal example; GPU execution uses an optional feature.

Today it supports 2D f32, one-cell compact halos, weighted Jacobi and an isotropic
five-point Laplacian. Single steps accept caller-supplied boundary samples;
repeated batches only preserve fixed Dirichlet boundaries. It doesn't decide
convergence or certify discretization error.

- Which boundary policy is the first missing one for your actual workload: periodic, Neumann, time-dependent Dirichlet, or something else?
- Does your halo live in one padded array, separate faces, or a strided subdomain? A sketch with dimensions would help.
- What blocks trying this on your cluster: CUDA/compiler versions, offline installs, scheduling, precision, or integration with existing arrays?

[Source and limitations](https://github.com/MannanSaood/gudra). One concrete
workload is enough; please avoid private datasets and machine identifiers.

## r/rust

Title: Feedback on Rust ownership for GPU stencil buffers: can the guarantee be understood?

I'm working on Gudra, an experimental source-only Rust library for 2D GPU
Jacobi/Laplacian stencils. In-place Jacobi writes can change values that neighboring
updates still need from the old iteration. The API uses a distinct haloed source
type, exclusively borrowed output, and consuming async buffer owners to make
that misuse unavailable to safe callers. No raw-device import/clone is exposed.

This is a narrow guarantee: it doesn't validate halo freshness, convergence,
compiler/driver internals or shared-GPU isolation. CPU tests include analytic
and f64 oracle fixtures; recorded single-GPU runs include real API compile-fail
checks and sanitizers. Multi-GPU validation is still pending. The README has a
CUDA-free runnable example and an alias rejection example.

- After reading the README, could you describe the prevented bug and one thing the API does not protect?
- Is moving the buffers into pending async work an understandable tradeoff, or would it block integration in your application?
- Where does the setup/documentation assume knowledge a Rust developer new to CUDA wouldn't have?

[README and evidence](https://github.com/MannanSaood/gudra). I'm asking for design
and documentation criticism, especially a concrete confusing passage or missing
workflow, rather than votes or downloads.

## NVIDIA Developer Forums

Title: Feedback on Jacobi buffer ownership and cluster deployment requirements in Rust/cuTile

I'm developing Gudra, an experimental Rust/cuTile 0.3.1 library for five-point
stencils. Separate old-grid and output allocations protect the public API from
an accidental in-place Jacobi update. It currently supports compact 2D f32 halos
and fixed-boundary double buffering. It trusts the backend/driver and aborts the
worker if completion cannot be proven; it doesn't promise context recovery.

A returned RTX 5060 Laptop / Arch / CUDA 13.4.92 run measured a warmed blocking
Laplacian call at 8.171-8.262 us for 256x256 versus 13.812-14.004 us in the scalar
CPU oracle; the 17x19 GPU case was much slower. These exclude JIT and transfers,
include launch/synchronization, and are not optimized-CPU or end-to-end claims.
The standalone two-GPU halo worker still needs hardware correctness, sanitizer
and P2P/staging/overlap evidence.

- Which ownership/lifetime bugs do you see around cancellation or buffer reuse in your own timestep code?
- What deployment prerequisite would prevent a trial on your cluster, and what diagnostic would make that failure clear?
- For halo overlap, which transfer/synchronization costs and same-host baseline should a credible report include?

[Project, methodology and limits](https://github.com/MannanSaood/gudra). Please
share a redacted workload description or synthetic reproducer; no GPU UUIDs,
private cluster paths or production logs are needed.

## After a response

Record a permalink and a concise evidence summary in [the feedback log](feedback.md).
Ask one focused follow-up if the workload is unclear. Use the report templates
for reproducible failures; move vulnerability details to private reporting.
Connect repeated independent requests to roadmap IDs and explain prioritization.
No reply or popularity count closes the target-user evaluation gate by itself.
