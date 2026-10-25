# Chat 11 handoff: documentation and community validation

Date: 2026-10-12. Active checkout: gudra. The pre-existing Chat 08-10 runtime,
prototype and evidence work is preserved. No commit, push, package publication,
release, settings change or community message was made. gudra-public is untouched.

## Delivered

README now starts with mixed old/new in-place Jacobi reads and the Rust API
boundary. It includes exact scoped guarantee/non-guarantees, a minimal CUDA-free
example, alias rejection, GPU invocation, numerical conventions, honest timing
summary, architecture/roadmap/contribution/security links. Setup/API/safety/results
navigation reconciles historical blockers with returned single-GPU evidence.

New guides: validation-status.md, benchmarks.md, architecture.md, roadmap.md,
troubleshooting.md and new-user-walkthrough.md. CONTRIBUTING.md and a short
project-specific CODE_OF_CONDUCT.md document solo-maintainer moderation and the
actual platform reporting routes. No nonexistent conduct inbox/team is advertised.
The maintainer should confirm enforcement responsibility before posting.

Community package: four tailored drafts in community/posts.md for cuTile
Discussions, Rust-SciComp Zulip, r/rust and NVIDIA Developer Forums; evidence-linked
feedback log/rubric and reporting/redaction checklist. Issue templates cover bug,
performance, safety-boundary and environment reports, plus private-report routing.
An Ideas discussion form and PR template are included. The Ideas form activates
only if repository Discussions is enabled and a matching Ideas category exists;
no repository settings were changed or hosted template behavior verified.

## Verified

See [performed clean-source walkthrough](../new-user-walkthrough.md): all ten
recorded commands succeeded, using Cargo 1.90 with Rust/rustdoc 1.89 and cached
dependencies. CPU debug/release each passed 28 tests and one doctest, three CPU
UI controls passed, examples and docs passed. Stable formatting/Clippy passed;
both README rustdoc examples passed. Documentation links, YAML/frontmatter and
new-package privacy checks are captured in documentation-qa.json. No local GPU
claim is made. Prior GPU/security evidence is explicitly bound to its source.

## Acceptance disposition

| Gate | Disposition |
|---|---|
| Protected failure understandable from README | Explained with runnable/compile-fail examples; actual target-user explanation not yet collected |
| Setup verified | Fresh source/empty target CPU path passed with documented workaround; first install/network/public candidate clone and local GPU remain unverified |
| Claims match numerical/security/benchmark evidence | Reconciled and revision-scoped; no universal safety, convergence, multi-GPU or production claim |
| No sensitive information in new examples/reports | Scoped scans and manual review; saved local paths redacted, report templates prohibit raw identities/private data |
| New-user walkthrough | Performed clean-source path, cached tools/dependencies disclosed; no independent reviewer represented |
| Community validation | Drafts ready, no posts/replies yet; feedback log has only internal/historical entries |

## Next maintainer actions

1. Make this exact documented source available through the repository or a reviewed source-only bundle; verify every draft's linked candidate before sending.
2. Confirm conduct enforcement ownership, current venue rules and category/topic, then publish selected drafts using the maintainer's own account. Do not send duplicate announcements.
3. Ask one scientific Rust/CUDA user to follow the README and explain the failure plus a non-guarantee; record the actual result, setup friction and consented evidence.
4. Record replies with evidence/permalinks; deduplicate independent workloads and connect repeated requests to roadmap IDs. Views/votes are reach only.
5. Keep two-GPU issue 3 and default-spin timestep characterization open. Their deferred status does not block this documentation package or establish production readiness.

This handoff is complete as a local documentation/outreach-preparation package.
The overall community-validation gate remains open pending real user evidence.
