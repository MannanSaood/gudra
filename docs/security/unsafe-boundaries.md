# Unsafe and execution boundary review

Gudra has **zero handwritten Rust unsafe blocks, unsafe functions, unsafe trait implementations, extern blocks or unchecked-index operations** in its library, examples and tests. `Cargo.toml` forbids unsafe code. This lint is not a proof about proc-macro expansion, dependencies, JIT code or CUDA.

The [dependency lexical inventory](dependency-unsafe-inventory.csv) records every line containing an unsafe token in the resolved dependency Rust sources, including comments, tests and macro templates. It is an enumeration, not an AST count or a claim that every dependency function was reviewed. The [dependency inventory](dependency-inventory.json) binds names/versions/checksums and build/native/proc-macro capabilities.

| Boundary | Assumption/control | Reviewed evidence and test status |
|---|---|---|
| Shape arithmetic and casts | Positive axes; checked halo/products/bytes; isize bytes, i32 axes/counts/stride; each grid axis <=65535 | `src/shape.rs`; overflow/zero/grid tests pass on CPU |
| Host reference indexing | Exact lengths first; index within initialized slices | `src/reference.rs`; independent oracle and atomic validation tests pass |
| Upload/allocation | f32 layout; validated contiguous counts; host Arc through copy terminal; private buffer/session | `cutile_impl.rs`; dependency DeviceBuffer/allocator/API paths reviewed; final GPU checks blocked locally |
| Directional views | Fixed offsets, halo stride width+2, no exposed view constructors | Five slices and metadata checks in `directional_views`; labeled view probes previously recorded on GPU |
| Generated launch | 16x16 partition; disjoint output tiles; correct rank/address space and partial-tile masks | `kernels.rs`, pinned macro launcher and kernel load paths reviewed; prior sanitizer evidence only |
| Context and threads | Arc session equality, device binding on entry and resumed poll; retained device/stream | `validate_step`; cuda-core runtime Send/Sync and cuda-async future/launch/reactor review; final UI/GPU rerun required |
| Readback | Device Arc retained until sync terminal returns; fresh host Vec initialized by completed transfer | Current explicit retained Arc repairs earlier issue; `073f7c6` remote Memcheck/Initcheck zero errors; failed sync separate |
| Borrowed execution error/unwind | Output marked uncertain before backend call; only success restores readiness | CPU injection tests pass; GPU invalidation test supplied; quiescence after driver error unproved |
| Async cancellation/drop | Outer frame owns buffers; child drop drains private stream | Normal-path source review; unpolled and ordinary lifecycle tests; forced-pending/failed-drain retention incomplete (SEC-004) |
| Resource admission | Atomic logical accounting, checked additions, RAII rollback, forgotten owners remain charged | CPU concurrent/overflow/exhaustion/unwind tests pass; new GPU wiring regression awaits execution |
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
