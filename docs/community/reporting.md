# Report data and privacy

Use synthetic data and the smallest reproducible grid. Before sharing, replace
home/cluster paths, usernames, hostnames, IPs, GPU UUIDs/serials, job/account IDs,
customer grids, private repository/attachment URLs and tokens with placeholders.
Keep consistent placeholders where identity relationships matter. Preserve public
source hashes, hardware model/capability, tool versions, shapes, commands, error
codes and numerical results. Do not paste `env`, nvidia-smi -q or whole cluster
configuration. Inspect logs and profiler captures locally before attaching them.
A secret scan is useful but cannot detect every private identifier.

For every report include:

- Public source revision and dirty status; candidate source digest when available; whether source came from a checkout or bundle.
- Exact command, features, debug/release profile, expected/actual result, minimized synthetic input/seed, shape and halo layout.
- OS/distribution/architecture, rustc and Cargo versions separately, cuTile and lockfile identity, relevant vendor changes.
- For GPU: model/CC, driver, nvcc/tileiras, Clang and loaded libclang version; CUDA path as a redacted placeholder if needed.
- Supported lane versus experimental lane, isolation assumptions, shared-host contention and resource limits. Never include credentials or raw service diagnostics sent to untrusted clients.

For performance add operator/iterations, h_squared/omega, timing scope, timer,
warmups, calls/sample, raw samples and medians/range, input correctness check,
allocation/transfer/JIT inclusion, default or explicit spin budget, power/clock
policy, same-host baseline and repetitions. Report CPU model too, without hostname.

For environment failures add the failing setup step and exit status plus selected
redacted diagnostics from `check-gpu-env.sh`; don't install arbitrary fixes without
recording them. For safety-boundary questions state whether safe Rust, unsafe/FFI,
external streams or imported storage were involved and what invariant you expected.
Suspected vulnerabilities go privately through SECURITY.md.

Native halo reviewers may retain detailed identity/topology in local evidence,
but public reports use device A/B and directed peer capabilities. Review any
full evidence transfer privately with the maintainer; raw runbook outputs are
not automatically suitable for a public issue.
