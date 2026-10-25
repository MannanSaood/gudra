# Evidence-linked roadmap

No deadlines or demand are inferred from stars, votes or download counts. A
request becomes a roadmap candidate with a concrete workload, linked evidence,
measurable outcome and explicit tradeoff. See the [feedback rubric](community/feedback.md).

| ID | Item and status | Evidence / next decision |
|---|---|---|
| R1 | Two-GPU native validation; open, does not block documentation | [Issue 3](https://github.com/MannanSaood/gudra/issues/3), [Chat 10](handoffs/10-multi-gpu-halo.md); validate P2P and staging, reference/overlap correctness, faults and sanitizers before claims |
| R2 | Default-spin timestep characterization; open | [Chat 09](handoffs/09-focused-stencils.md), [timestep measurements](timestep-runtime.md); same-host comparison including timing scope, no promised async speedup |
| R3 | Target-user comprehension and cluster setup trial; open | [walkthrough](new-user-walkthrough.md), [draft questions](community/posts.md); user explains old/new mixing, completes a clean setup and reports a deployment blocker |
| R4 | Periodic/Neumann/evolving boundaries and real halo layouts; discovery only | [current operator limits](operators.md); require linked workload examples and repeated independent requests before choosing an API |
| R5 | 3D/f64/anisotropic or additional operators; deferred | [ADR 009](adr/009-focused-operators.md); validate scientific use case, tolerance and performance cost first |
| R6 | CUDA graphs; deferred | [ADR 008c](adr/008c-graphs.md); require measured launch bottleneck and safe ownership integration |

For repeated requests, add feedback IDs and public issue/permalink evidence to the
relevant row, count independent workloads (deduplicate reposts), and describe
why the existing API fails. Conflicting requests stay visible. A maintainer
records accepted/deferred/rejected decisions and revisits them when evidence changes.
Publication remains a separate owner decision under [the release process](release-process.md).
