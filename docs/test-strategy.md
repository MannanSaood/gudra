# Verification lanes and property strategy

Run from the repository root. CPU tests require only Rust; the UI runner adds
Python 3's standard library. No new Cargo dependency or CPU fallback is introduced.

```sh
cargo test --locked --offline
python3 scripts/check-ui.py
cargo fmt --check
cargo clippy --locked --offline --all-targets -- -D warnings
cargo doc --locked --offline --no-deps
```

The GPU compile-only lane needs the CUDA/cuTile development toolchain but does
not launch a GPU:

```sh
cargo check --locked --features gpu --all-targets
python3 scripts/check-ui.py --gpu
```

On the [supported Linux/CUDA environment](setup.md), the full required gate is:

```sh
bash scripts/verify-gpu.sh
```

It runs GPU linting and real API UI tests, doctests, **both** library tests and
`gpu_jacobi`, five process repetitions with `CUDA_ASYNC_SPIN_BUDGET_US=0`, and
memcheck/initcheck against the actual test executables. A missing prerequisite or
any failed test exits nonzero. There are no ignored tests or hardware-based skips.
The directional probe lives in library tests and must not be omitted by running
only `--test gpu_jacobi`. The [negative control](../fixtures/README.md) is separate;
its numerical mismatches are observations, never an acceptance threshold.

## Coverage matrix

| Concern | Cases and assertion |
|---|---|
| Baseline formula | Exact literal 1x1 answer 6.125; six independent 2x3 answers |
| Shapes | 1x1, 1x19, 19x1, 2x3, 3x2, 15x17, 16x16, 17x19, 32x48, 63x65, 129x257, 257x263 |
| CPU oracle independence | 12 shapes x 5 coefficient pairs, separately indexed `f64` windows |
| Structural shape properties | Every pair of axes 1..=65 (4,225), no device allocation |
| Manufactured solution | 65 quadratic grids, RHS=4, h²=1; exact preservation at omega=-2 |
| Replayable finite data | Fixed LCG seed 0x5eed0005; 64 seeds/shapes for zero-weight identity; RHS and halo vary |
| Edges/corners | NaN halo corners, poison-filled output; all interior cells compared, including partial tiles |
| Directional orientation | Each actual production view copied separately; sentinel `4096*row + 3*column`; 1x1, 2x3, 17x19, 33x47 |
| Coefficients | Negative, zero, one, greater than one, and finite extremes; reject NaN/infinity/negative h² |
| Nonfinite policy | NaN/±infinity in each used source location and RHS; zero times nonfinite; opposing infinities; finite overflow; extreme coefficients on zero data |
| Invalid geometry | Zero axes; every checked add/product/bytes overflow; isize/i32 and tile-grid limits |
| Invalid buffers | Short/long host slices; equal-area wrong RHS/output extents; every foreign-session role |
| Ownership | Real API compile success controls plus E0502/E0499/E0308/E0382/E0616 rejection fixtures |
| Lifecycle | Validation preserves sentinel output; error/unwind invalidates borrowed output; no read/reuse of invalid output; moved async owners; cross-thread completion; unpolled/first-polled drop |
| Schedule sampling | Three fresh output repetitions per GPU shape, three reuses per output; ten drop cycles per test; five independent test-process runs |

`tests/support/mod.rs` defines the reproducible generator, cases, and classification
aware tolerance assertion. Its NaN policy cannot accidentally pass an unexpected
NaN by using a comparison such as `if error > tolerance` alone.

The default property strategy deliberately avoids an external fuzz/property-test
dependency. Bounded exhaustive shape checks and deterministic arithmetic fixtures
run quickly without GPU hardware. Expand a failed seed into a minimal literal
regression before widening the generator. Future fuzzing should bias axes toward
0/1/15/16/17, checked-size limits, equal-area transposes, and one exceptional float
at a time; never allocate from unchecked fuzzed sizes.

For mutation testing on the GPU host: swap north/south in `directional_views`
(probe must fail even if Jacobi passes), change the RHS sign (oracle must fail),
omit the last tile/store (poison-output test must fail), or bypass invalidation
(lifecycle test must fail). These mutations are a strategy, not locally recorded
GPU results. Do not commit mutations or loosen tolerance just to hide failures.

Forced-pending cancellation, forgotten-future leak tests, driver failure injection,
and 32-bit execution remain separate unverified cases. Run fault experiments in
isolated processes, retain environment/test logs, and distinguish attempted,
passed, failed, and blocked checks in [verification results](verification-results.md).
