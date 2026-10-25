# ADR 008a: Owned lazy asynchronous batches

Status: accepted; supplied Chat 08 Arch experimental GPU validation passed.
See the evidence scope in ../handoffs/08-timestep-runtime.md.

Use an opaque `Future<Output = Result<DoubleGrid>> + Send + 'static`, with immediate
admission and lazy first submission. The future owns current, next, RHS, session,
and an operation reservation. Each child DeviceFuture borrows only inside this
owned frame; completion precedes swap and publication. The explicit ownership
transition does not require a public `Grid<State>` hierarchy. No executor crate,
background thread, custom polling, or custom unsafe code is introduced.

A borrowed public future would need a separate proof against forgotten futures
and early borrow release. A detached worker needs queue limits, shutdown/join and
panic semantics. Both add mechanisms already handled by existing owned terminals.

Cancellation is drop: unpolled means no submit; pending means drain then discard.
All recoverable errors consume the state. Unknown completion aborts under the
existing backend policy. This sacrifices rollback and nonblocking cancellation
for a narrow, auditable ownership contract. Observation is a completed readback
between bounded batches. Large batches may monopolize a poll if children complete
immediately; callers should use bounded step counts.
