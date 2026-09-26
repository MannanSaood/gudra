# Unsafe and execution boundary review

## 2026-09-25 remediation status

Gudra's supported GPU facade now retains backend allocations, foreign owners,
session state, and host transfer destinations while execution may still be in
flight. A successful synchronization, callback, query, or cancellation drain
releases the quarantine. An error or unwind that cannot prove completion aborts
the worker before an owner may be destroyed or reused. Allocation ownership is
established before fallible host-to-device copies, closing the prior physical
allocation and logical-accounting gap.

Deterministic CPU tests establish state transitions and owner retention,
including the corrected asynchronous handoff. The clean pinned final revision
also passed drop-in-flight, forced CUDA 719 fail-stop, repeated execution, and
Compute Sanitizer memcheck/initcheck on the disposable GPU worker. Gudra keeps backend
fields private and does not expose raw operations, graph construction, SIMT
APIs, or cuTile tensor constructors. Applications that separately depend on
`cutile` or `cuda-async` must review and support those additional boundaries.

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

Gudra has **zero handwritten Rust unsafe blocks, unsafe functions, unsafe trait implementations, extern blocks or unchecked-index operations** in its library, examples and tests. `Cargo.toml` forbids unsafe code. This lint is not a proof about proc-macro expansion, dependencies, JIT code or CUDA.

The [dependency lexical inventory](dependency-unsafe-inventory.csv) records every line containing an unsafe token in the resolved dependency Rust sources, including comments, tests and macro templates. It is an enumeration, not an AST count or a claim that every dependency function was reviewed. The [dependency inventory](dependency-inventory.json) binds names/versions/checksums and build/native/proc-macro capabilities.

| Boundary | Assumption/control | Reviewed evidence and test status |
|---|---|---|
| Shape arithmetic and casts | Positive axes; checked halo/products/bytes; isize bytes, i32 axes/counts/stride; each grid axis <=65535 | `src/shape.rs`; overflow/zero/grid tests pass on CPU |
| Host reference indexing | Exact lengths first; index within initialized slices | `src/reference.rs`; independent oracle and atomic validation tests pass |
| Upload/allocation | f32 layout; validated contiguous counts; host Arc through copy terminal; private buffer/session | `cutile_impl.rs`; dependency paths reviewed; final allocation, budget and sanitizer GPU checks pass |
| Directional views | Fixed offsets, halo stride width+2, no exposed view constructors | Five slices and metadata checks in `directional_views`; labeled view probes previously recorded on GPU |
| Generated launch | 16x16 partition; disjoint output tiles; correct rank/address space and partial-tile masks | `kernels.rs`, pinned launcher and load paths reviewed; final memcheck/initcheck and partial-tile tests pass |
| Context and threads | Arc session equality, device binding on entry and resumed poll; retained device/stream | Runtime review plus final cross-thread owned-future, session rejection and repeated GPU tests pass |
| Readback | Device Arc retained until sync terminal returns; fresh host Vec initialized by completed transfer | Current explicit retained Arc repairs earlier issue; `073f7c6` remote Memcheck/Initcheck zero errors; failed sync separate |
| Borrowed execution error/unwind | Output marked uncertain before backend call; only success restores readiness; undrainable fault aborts worker | CPU seams, GPU invalidation and CUDA 719 fail-stop test pass |
| Async cancellation/drop | Outer frame owns buffers; child drop drains private stream or fail-stops before release | Unpolled, forced-pending, drop-in-flight and failed-drain fault tests pass; SEC-004 closed |
| Resource admission | Atomic logical accounting, checked additions, RAII rollback, forgotten owners remain charged | CPU and final GPU allocation/exhaustion/release tests pass |
| JIT subprocess/artifact load | Trusted compiler/native libraries; authenticated writer boundary before unsafe cuModuleLoadData | Transient files SEC-001; private Linux launcher added; no adversarial image executed |
| Native negative control | CUDA C++ fixture has explicit f32 pointers and manual launch; constant bounds and initialized storage | `fixtures/in_place_jacobi.cu`; intentional in-place global race, excluded from product and security-vulnerability count |

Lexical counts across the eight CUDA/cuTile 0.3.1 crates' `src` trees from the focused review:

| Crate | unsafe impl lines | unsafe block lines | unsafe fn lines |
|---|---:|---:|---:|
| cutile | 5 | 57 | 55 |
| cutile-compiler | 0 | 7 | 0 |
| cutile-ir | 0 | 0 | 0 |
| cutile-macro | 0 | 3 | 2 |
| cuda-core | 38 | 222 | 133 |
| cuda-async | 25 | 69 | 54 |
| cuda-bindings | 0 | 11 | 8 |
| cuda-core-derive | 1 | 0 | 0 |

Reviewed reached manual impl groups include cuda-core Device/MemPool/Stream/Event/Module/Function Send+Sync (`runtime.rs:90-91,422-423,580-581,597-598,935-936,992-993`); DeviceBuffer Send/Sync and DevicePointer Send (`device_buffer.rs:69,125-126`); AsyncKernelLaunch Send and aligned argument arena (`launch.rs:41,70-100`); reactor pinned CudaFlags Send/Sync (`reactor.rs:64-65`); SlotCell Sync and acquire/release reuse (`slot_table.rs:43,118-202`); borrowed partition/readback Send (`tile_kernel.rs:1561,1633,1942`); composition closure Send bounds; generated DeviceCopy with checked field types.

Fully read dependency files: compiler jit_cache/specialization/build; cuda-async device_future/device_buffer/launch/reactor; cuda-bindings build/dyn_load; cuda-core build; cuda-core-derive lib/device_copy. Production portions of device_context/slot_table and relevant runtime/API/tensor/operation/macro/JIT portions were traced. SIMT, VMM, graph, unrelated kernels and generated native bindings were inventoried but not exhaustively reviewed. Proprietary tileiras/driver internals and machine code were unavailable.

**Acceptance:** the first-party unsafe enumeration is complete. The user's stronger requirement that every unsafe boundary be documented and tested or removed is **not yet met** for generated/dependency fault paths. No unrun GPU or sanitizer test is counted as passing.
