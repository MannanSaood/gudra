# Evidence and claim boundaries

Updated 2026-10-12. Evidence is revision-scoped; a pass never transfers automatically
to a changed runtime or dependency. This index supersedes older present-tense
status text in historical reports without changing their original observations.

| Surface | Evidence | What it supports / leaves open |
|---|---|---|
| CPU reference and ownership | [Chat 11 clean-source walkthrough](new-user-walkthrough.md), [CPU tests](../tests/) | Runnable examples, numerical fixtures, alias diagnostics on stated toolchain; no GPU claim |
| Reviewed base runtime/security | [Chat 07 closure](handoffs/07-release-gate-closure.md), [audit](security/audit-report.md), [verification](verification-results.md) | Technical gate for c5f3322 / runtime 4debea7, JIT/completion/fault seams and sanitizer evidence; no approval to publish or proof of tenant isolation |
| Fixed-boundary timestep ownership | [Chat 08](handoffs/08-timestep-runtime.md) | Returned single-GPU batch, cancellation, numerical and sanitizer results on that source; no batch-overlap promise |
| Laplacian and current single-GPU additions | [Chat 09 final section](handoffs/09-focused-stencils.md), [review](evidence/chat09/returned-v2/review.json) | Manifest-matched user-supplied 28a9100 results: 26 commands, five repetitions, 12 UI controls, six zero-error sanitizer summaries; Arch experimental lane, not an independently attested remote tree |
| Native halo exchange | [Chat 10](handoffs/10-multi-gpu-halo.md), [failure injection](handoffs/10-halo-failure-injection.md), [issue 3](https://github.com/MannanSaood/gudra/issues/3) | Host coordinator/failure checks and scoped source review; CUDA compilation, two-device correctness, P2P/staging, faults, sanitizers and overlap profiling remain unverified |
| Comprehension and demand | [Feedback log](community/feedback.md) | No target-user comprehension interview or new community feedback yet; drafts and self-review are not user validation |

Security review of the base does not automatically certify later features. The
native prototype requires fresh review before production use. Re-run GPU checks
if runtime, dependencies, compiler or policy change. Documentation checks do not
reopen already satisfied scoped hardware checks merely because time has passed.
Historical technical readiness is separate from the unpublished current development
checkout and from explicit owner authorization to release.
