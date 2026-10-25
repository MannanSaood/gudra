---
name: "Performance report"
about: "A measured regression or workload bottleneck"
title: ""
labels: ""
assignees: ""
---

## Workload and baseline
Operator, layout, dimensions, spacing/weight, iterations, same-host baseline,
correctness assertion, expected/actual timing and relevant prior revision.

## Measurement data
Warmups, samples and calls/sample, timer, all samples/median/range, JIT/transfer/
allocation/synchronization inclusion, spin budget, CPU model, GPU power/clock policy,
shared-host load and repeats. Attach a redacted command/output; avoid cross-host speedup.

## Revision and environment
Public source revision/dirty status, OS/architecture, rustc/Cargo, features and
profile. For GPU include model/CC, driver, nvcc/tileiras and Clang/libclang versions.
See [report data](../../docs/community/reporting.md) for the full checklist.

## Privacy check
Use synthetic data. Redact usernames, hostnames, absolute cluster/home paths,
GPU UUIDs/serials, tokens and private links before attaching logs.
Suspected vulnerabilities belong in [private reporting](https://github.com/MannanSaood/gudra/security/advisories/new),
not this public issue; follow [SECURITY.md](../../SECURITY.md) if unavailable.
