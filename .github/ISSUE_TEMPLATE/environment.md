---
name: "Environment report"
about: "A blocked installation, build or cluster trial"
title: ""
labels: ""
assignees: ""
---

## First failing step
Exact setup command and exit status; expected and actual diagnostic.

## Deployment context
Supported/experimental lane, native/container/batch scheduler, offline access,
linker availability, chosen toolkit, selected redacted checker lines, and fixes
already tried. Do not paste environment variables or full machine inventories.

## Revision and environment
Public source revision/dirty status, OS/architecture, rustc/Cargo, features and
profile. For GPU include model/CC, driver, nvcc/tileiras and Clang/libclang versions.
See [report data](../../docs/community/reporting.md) for the full checklist.

## Privacy check
Use synthetic data. Redact usernames, hostnames, absolute cluster/home paths,
GPU UUIDs/serials, tokens and private links before attaching logs.
Suspected vulnerabilities belong in [private reporting](https://github.com/MannanSaood/gudra/security/advisories/new),
not this public issue; follow [SECURITY.md](../../SECURITY.md) if unavailable.
