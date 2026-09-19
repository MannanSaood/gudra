# Chat 05 verification results

**Acceptance gate: failed Memcheck; readback lifetime patch awaits GPU verification.**
The original local results below remain a historical record. A friend supplied
successful initial GPU results on Arch; see the follow-up here. CPU results alone
are not treated as GPU evidence.

## Readback sanitizer follow-up

After `71f2486`, the user reported GPU Clippy success and supplied
[the complete closure log](evidence/chat05-gpu-before-readback-fix.log).
All five repetitions passed (8 library + 7 GPU integration tests each). The first
Memcheck invocation then reported **21 use-after-free errors** during readback.
Initcheck and the integration executable's sanitizer runs were not reached.
Passing numerical assertions do not override those errors.

The patch retains a private tensor owner until the copy stream's synchronization
returns, and adds a kernel-independent readback regression plus a portable
sanitizer runner. [Investigation, exact rerun commands, and remaining limits](readback-lifetime.md)
describe why this is a candidate repair rather than verified closure.

## Arch follow-up (user-supplied output)

Reported environment: Arch Linux x86_64, RTX 5060 Laptop GPU (compute capability
12.0), driver 615.71.09, CUDA/nvcc/tileiras 13.4, Rust 1.89.0, Clang 22.1.8.
The loaded libclang version and checkout commit hash were not captured in the
submitted test log. These are remote observations supplied by the user, not
GPU runs performed on this local host.

The initial run passed all eight UI fixtures, three doctests, eight library tests
(including actual directional views and invalidated-field rejection), seven GPU
integration tests, and the 323-cell Poisson example (reported max absolute error
zero). The subsequent closure command stopped at GPU-feature Clippy: single-letter
kernel names, wildcard imports, and exclusive ranges triggered pedantic lints.
No process repetitions or sanitizer checks ran in that command.

The follow-up patch renames kernel locals without changing arithmetic, scopes
wildcard-import exceptions to the cuTile DSL imports, and allows `range_plus_one`
only on the view constructor because `Tensor::slice` requires `Range<usize>`.
Local CPU linting, formatting, and diff checks pass. GPU lint and execution of
the patched source must be rerun; prior numerical results do not certify the
new revision. The strict Ubuntu/CUDA 13.3 checker still does not accept this Arch
environment, so its failure is separate from the observed runtime results.

## Original local environment

Recorded 2026-09-19 against base `7166630` plus the Chat 05 working-tree changes.
Host: Windows x86_64. Rust compiler/rustdoc: 1.89.0 (29483883e, LLVM 20.1.7).
Cargo: 1.90.0 (840b83a10). Formatting/linting: installed stable toolchain 1.90.0.
cuTile remains locked to 0.3.1; Cargo dependencies and lockfile were unchanged.
No `nvcc`, `nvidia-smi`, or `compute-sanitizer` was available on PATH. The actual
dependency build confirmed that no CUDA toolkit was found in its default paths.

## Observed results

| Check actually attempted | Result | Evidence |
|---|---|---|
| Plain `cargo test --locked --offline` | Blocked: pinned installation lacks its Cargo binary | Rust 1.89 compiler exists; Cargo shim reports component not applicable |
| Cargo stable + explicit Rust 1.89 compiler/rustdoc, CPU debug tests | PASS: 18 tests + 1 doctest, none ignored | [raw log](evidence/chat05-cpu-debug.txt) |
| Same compiler, CPU release tests | PASS: 18 tests + 1 doctest | [raw log](evidence/chat05-cpu-release.txt) |
| CPU UI runner, exact current library artifact | PASS: positive control; E0502 alias rejection | [raw log](evidence/chat05-ui-cpu.txt) |
| `cargo +stable clippy --locked --offline --all-targets -- -D warnings` | PASS on stable 1.90; GPU disabled | [raw log](evidence/chat05-clippy.txt) |
| `cargo +stable fmt --all -- --check` | PASS | Exit 0 |
| Cargo stable + Rust 1.89 `doc --locked --offline --no-deps` | PASS | [raw log](evidence/chat05-docs.txt) |
| Python UI runner syntax and Git Bash `bash -n scripts/verify-gpu.sh` | PASS | Exit 0; syntax only for GPU script |
| `git diff --check` | PASS | Exit 0 |
| `python scripts/check-ui.py --gpu` with Rust 1.89 | BLOCKED, exit 1 in `cuda-bindings` before Gudra GPU type-checking | [raw log](evidence/chat05-ui-gpu-blocked.txt) |

The GPU runner did not count dependency failure as a successful rejection. No
GPU fixture has an observed expected diagnostic yet. GPU-feature clippy, tests,
doctests, sanitizer runs, CUDA C++ compilation, and negative-control observations
were **not run**. Five GPU process repetitions are supplied in the runner but are
not reported as executed. CPU debug/release reruns check optimization behavior;
they do not sample GPU scheduling.

## Exact local workaround

No toolchain files or machine installations were changed. In PowerShell:

```powershell
$env:RUSTC = "$env:USERPROFILE\.rustup\toolchains\1.89.0-x86_64-pc-windows-msvc\bin\rustc.exe"
$env:RUSTDOC = "$env:USERPROFILE\.rustup\toolchains\1.89.0-x86_64-pc-windows-msvc\bin\rustdoc.exe"
$env:CARGO = "$env:USERPROFILE\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe"
cargo +stable test --locked --offline
cargo +stable test --locked --offline --release
cargo +stable doc --locked --offline --no-deps
python scripts/check-ui.py
python scripts/check-ui.py --gpu  # fails visibly until the GPU build prerequisites exist
$env:RUSTC = $null
$env:RUSTDOC = $null
cargo +stable clippy --locked --offline --all-targets -- -D warnings
cargo +stable fmt --all -- --check
```

The GPU environment in `docs/setup.md` is a target configuration, not a measured
environment from this session. Local CPU tests need no CUDA library, driver,
device, or fake backend. Pinning lint verification to 1.89 on a fully installed
toolchain remains part of the Linux closure gate.

## Gate disposition

| Acceptance criterion | Status |
|---|---|
| Advertised device aliasing misuse fails before execution | Real API UI fixtures and controls supplied; **blocked**, actual GPU crate not built |
| Supported numerical cases match CPU oracle | CPU oracle independently checked; **GPU comparison pending** |
| Unsupported cases fail explicitly | CPU geometry/parameter/length cases pass; GPU shape/session/invalidated-field checks pending |
| Guarantees and non-guarantees documented | Complete: [safety model](safety-model.md) |

The implementation change discovered during verification is conservative output
invalidation after a failed or unwound borrowed backend execution. CPU state tests
pass. The wiring into the actual GPU API must still pass its GPU unit test; it
does not establish safety after a failed device drain.

## Closure required

On the pinned supported Linux/CUDA host, run `bash scripts/verify-gpu.sh` and the
[negative-control commands](../fixtures/README.md). Save compiler/driver/toolkit/
GPU identity and raw output, then update this table from observations. Treat any
unexpected UI code, view orientation failure, nonfinite-class mismatch, sanitizer
failure, or numerical disagreement as a regression to investigate. Do not widen
tolerance or claim a pass because the failing prerequisite is unavailable.

Coverage details and follow-up properties are in [the test strategy](test-strategy.md).
