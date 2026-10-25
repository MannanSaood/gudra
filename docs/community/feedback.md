# Feedback log and prioritization

Updated 2026-10-12. No new community posts have been sent by this documentation
work and no replies have been collected. Prepared questions are hypotheses.
Stars, upvotes, views and downloads measure reach; they are not evidence of a
scientific workload, adoption, correctness or product value.

| ID | Source / date | Evidence and independent workload | Finding | Status / roadmap |
|---|---|---|---|---|
| F0 | [Existing issue 3](https://github.com/MannanSaood/gudra/issues/3), 2026-10-12 | Maintainer hardware-validation request, not a target-user reply | Two-GPU evidence absent | Open; R1 |
| F1 | [Clean-source walkthrough](../new-user-walkthrough.md), 2026-10-12 | Maintainer-agent execution, not independent user | Pinned Cargo incomplete locally; diagnostic/workaround needed | Documented; R3 |

## Intake and evidence

For each reply assign an ID; record date, venue and public permalink (or an
anonymized private-evidence reference kept outside this repo), user's role/workload,
layout, boundaries, scale, deployment environment, actual blocker, suggested
change, reproduction, confidence, and next action. Paraphrase respectfully;
obtain consent before reproducing private statements. Retain contradictory
feedback and failed attempts. Link replies across venues instead of counting a
repost or the same person twice. Do not invent quotes or treat no replies as rejection.

## Prioritization rubric

Triage plausible safety/correctness failures first; handle vulnerability details
privately regardless of popularity. For other requests record these dimensions
separately (0-2 each), plus engineering cost and risk (small/medium/large):

| Dimension | 0 | 1 | 2 |
|---|---|---|---|
| Workload evidence | Abstract wish | Specific workload described | Reproducer, data or measured blocked workflow |
| Impact | Cosmetic/preference | Workaround costly | Incorrect science or deployment blocked |
| Recurrence | One uncorroborated request | Two independent workloads | Three or more independent workloads |
| Fit | Broad new framework | Adjacent to current scope | Directly improves current protected workflow |
| Confidence | Untested hypothesis | Source/diagnostic evidence | Independently reproduced result |

Scores organize discussion, not automatic decisions. A single high-impact
reproducer can outrank many low-detail requests. Record maintainer rationale,
accept/defer/reject decision, uncertainty, owner and verification target. When
requests recur, update [the roadmap](../roadmap.md) with feedback IDs, links,
independent workload count, concrete outcome and cost; preserve the evidence
chain in the eventual issue/PR. Review after each substantive response and after
the initial outreach round; no scheduled automation is implied.

## First target-user session

Ask a scientific Rust/CUDA developer to read the README unaided, then explain
why old/new mixing changes Jacobi, which alias the API rejects, and one thing
it does not protect. Have them follow setup and run the example, recording
first failure, commands, elapsed effort and the intervention needed. Ask for a
real layout/boundary and cluster blocker. Record actual answers and consented
anonymized evidence. This interview remains open; self-review cannot close it.
