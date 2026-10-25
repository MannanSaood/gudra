---
name: "Bug report"
about: "A reproducible wrong result or API failure"
title: ""
labels: ""
assignees: ""
---

## Expected and actual behavior
Include the failing assertion/error code and whether results are finite.

## Minimal reproducer
Exact command, synthetic grid/seed, shape, halo/boundary policy, coefficients,
iterations and a CPU-reference comparison. Attach only redacted relevant output.

## Revision and environment
Public source revision/dirty status, OS/architecture, rustc/Cargo, features and
profile. For GPU include model/CC, driver, nvcc/tileiras and Clang/libclang versions.
See [report data](../../docs/community/reporting.md) for the full checklist.

## Privacy check
Use synthetic data. Redact usernames, hostnames, absolute cluster/home paths,
GPU UUIDs/serials, tokens and private links before attaching logs.
Suspected vulnerabilities belong in [private reporting](https://github.com/MannanSaood/gudra/security/advisories/new),
not this public issue; follow [SECURITY.md](../../SECURITY.md) if unavailable.
