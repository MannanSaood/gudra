# Benchmark methodology and honest results

The harness is [stencil_bench.rs](../examples/stencil_bench.rs). In release mode
it evaluates the five-point Laplacian on 17x19, 256x256 and 257x263 interiors,
for squared spacing 0.25, 1 and 4. Inputs are deterministic `(index % 31)/16`.
Output is allocated once. Each configuration has one warm call, then seven
wall-clock samples of 100 calls; sort the per-call averages and report the fourth.
CPU uses black_box. GPU times the blocking public API: host launch, kernel and
synchronization are included; JIT, upload, initialization and final readback are
excluded. GPU output is compared to the CPU fixture tolerance after timing.
These are medians of batch averages, not individual-call p50 or p99 latency.

```sh
cargo run --locked --release --example stencil_bench
python3 scripts/run-private-gpu.py -- cargo run --locked --release --features gpu --example stencil_bench
```

Do not compare a debug build with these release numbers. Keep the source revision,
lockfile, toolchain, feature set, spin budget, input, and timing scope with results.

## Returned single-worker sample

[All nine CPU/GPU pairs](evidence/chat09/returned-v2/benchmark.json) and
[full evidence](handoffs/09-focused-stencils.md): revision
28a9100000f5ab38154410ebdb4e8c327869e82f, 2026-10-03, Linux x86_64 Arch
experimental lane; Rust/Cargo 1.89.0, RTX 5060 Laptop GPU CC 12.0, driver
615.71.09, CUDA/tileiras 13.4.92. Spin budget unset (default).

| Interior | CPU us, h^2=0.25 / 1 / 4 | GPU blocking us, h^2=0.25 / 1 / 4 |
|---|---:|---:|
| 17x19 | 0.267 / 0.199 / 0.190 | 7.113 / 7.108 / 7.094 |
| 256x256 | 14.004 / 13.812 / 13.963 | 8.171 / 8.258 / 8.262 |
| 257x263 | 13.896 / 13.794 / 13.839 | 8.613 / 8.497 / 8.648 |

Small grids are launch dominated. Larger sampled grids show 1.60-1.71x lower
latency than this worker's scalar oracle. There is no optimized SIMD/threaded
CPU baseline, end-to-end speedup, confidence interval or general performance
threshold. CPU model, power/clock state and contention were not recorded. This
missing context limits comparability. User-supplied manifests/logs are internally
consistent; the remote Git tree was not independently retrieved or attested.

[Local Windows CPU sample](evidence/chat09/benchmark.json) used Rust 1.90.0 and
is a separate host/run. Do not calculate cross-host speedup from it.
[Timestep results](timestep-runtime.md) use a different scalar-tile kernel and
zero spin budget: 819 us blocking versus 1310 us owned async for that recorded
workload. They do not establish that async is faster or characterize default-spin
batches. Native halo host timings provide no GPU overlap evidence.

## Reproducible performance reports

Include all data in the [report checklist](community/reporting.md), all samples
when available, warmups, repetitions, median/range, timer type, correctness checks,
and exactly which costs were timed. Add separate cold/JIT and transfer-inclusive
measurements when they matter to your deployment. Report power/clock policy and
shared-host load without disclosing machine identifiers. Compare on the same
host against a relevant optimized baseline. Never silently discard outliers or
widen correctness tolerance to make a run pass.
