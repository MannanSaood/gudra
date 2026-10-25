---
name: "Safety boundary question"
about: "Clarify an invariant or unsupported workflow; vulnerabilities go privately"
title: ""
labels: ""
assignees: ""
---

## Expected protection and workflow
Which API invariant did you expect? Which owners/borrows and completion state?
Does the example use only safe Gudra APIs, or unsafe/FFI/external storage/streams?

## Minimal non-sensitive example
Exact diagnostic and a synthetic layout. If this may expose a vulnerability,
stop and use the private channel rather than posting exploitation details.

## Revision and environment
Public source revision/dirty status, OS/architecture, rustc/Cargo, features and
profile. For GPU include model/CC, driver, nvcc/tileiras and Clang/libclang versions.
See [report data](../../docs/community/reporting.md) for the full checklist.

## Privacy check
Use synthetic data. Redact usernames, hostnames, absolute cluster/home paths,
GPU UUIDs/serials, tokens and private links before attaching logs.
Suspected vulnerabilities belong in [private reporting](https://github.com/MannanSaood/gudra/security/advisories/new),
not this public issue; follow [SECURITY.md](../../SECURITY.md) if unavailable.
