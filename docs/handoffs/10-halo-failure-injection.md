# Chat 10 failure-injection and correctness report

Date: 2026-10-03. Tested implementation: standalone `prototypes/halo` CPU
protocol model, Windows x86_64, stable Rust/Cargo 1.90.0. No CUDA execution.

| Failure or condition | Test and observed result |
|---|---|
| Event reused at the same slot in the next timestep | `old_event_same_slot_different_timestep`: old full descriptor rejected |
| Previous epoch / previous buffer generation | `mismatches_poison_all_outputs`: mismatch, entire batch poisoned |
| Swapped direction / wrong neighbor | Same test: rejected before halo installation |
| Foreign simulation/device/context/buffer/source generation/streams/event | Same test: rejected, next progress and finish return Poisoned |
| Duplicate completion | Same test: Duplicate, then Poisoned |
| Peer unavailable | `admission_and_fallback` selects HostStaged; schedule stress matches reference |
| Copy failure after one successful receipt | `partial_copy_failure_and_partial_launch_discard`: no readiness/output returned |
| Device 1 launch failure after device 0 compute | Same test: consumed batch returns Launch; no generation returned |
| Cancellation | Fault::Cancel returns Cancelled, then Poisoned; no readiness returned |
| Pending drop on another host thread | `timeout_pending_drop_and_epoch_overflow`: safe CPU-owner discard, thread joins |
| Stalled peer / deadline | Same test: two pending ticks then Timeout, finish Poisoned |
| Pending finish | Same test: Pending, owned batch consumed/discarded |
| Epoch overflow | Same test: exchange rejected before submission |
| Odd/non-divisible sizes and thin slabs | `reference_and_schedule_stress`: exact full-domain reference equality |
| Concurrent independent simulations | Two OS threads with different inputs/transports match respective references; cross-session receipt rejected |
| Delayed transfers / randomized arrival | Seeded 1..32 schedules, reverse tie visitation; exact reference equality |
| Corrupted installed halo | `readiness_rechecks_halos_and_overlap_order`: readiness revalidation rejects it |
| Boundary before halos / observing old owner | Two compile-fail doctests reject safe API misuse; positive doctest exercises valid transition |
| Interior and boundary ordering | Trace assertions verify reference interior after all copies; boundary after copies AND interior |
| Admission caps | Invalid slab counts, >64 devices and excessive scheduler deadline explicitly rejected |

The stress matrix has seven lengths (2,3,5,7,17,31,65), all device counts
2..min(N,5), 32 seeds, two overlap modes and two transports: 2,944 runs of
12 timesteps = 35,328 checked generations per build profile. Debug and release
each passed all eight unit tests. Both compile-fail controls and the positive
control pass. Clippy all-targets with warnings denied passes; no dependencies
or unsafe code were introduced. Raw outputs are under `docs/evidence/chat10`.
Checksums identify the source/test files in `source-sha256.json`.

Failure injection targets modeled control flow and provenance. It does not
exercise physical DMA, driver events, P2P enablement, pinned-host staging,
device faults, kernel JIT, Compute Sanitizer, CUDA context destruction or stalled
hardware. Dropping CPU vectors proves nothing about native stream-drain safety.
Randomization covers arrival permutations from seeded delays with fixed tie order;
it is not an exhaustive concurrent-memory model checker. Inputs are finite in
the reference matrix; no claim of convergence or nonfinite equivalence is made.

Timeline seed 42, N=17, D=3: reference Peer finishes at tick 10, modeled overlap
at 9; reference HostStaged at 18, modeled overlap at 17. All copies and the
readiness synchronization gate are included. These assigned-tick makespans are
illustrations of the intended schedule, **not measured speedups**.

Before native acceptance, repeat the same failure cases against actual private
CUDA receipt owners and at least two discovered devices. Include directed peer
unavailability, mixed capability pairs, partial copy/launch fault, event recycle,
pending cancellation/drop, watchdog timeout, supervisor worker disposal and
sanitizer/profiler evidence. No public release gate is closed by this model.

## Native worker follow-up

The standalone `prototypes/halo-native` coordinator now calls actual CUDA through
`cuda_backend.cuh`. CUDA compilation/execution are unavailable on this host.
The same coordinator compiled locally with MSVC 19.40 using C++17, /W4 /WX,
/O2 and strict floating point against a queued host backend. The final supervised
run is `docs/evidence/chat10/native-final/halo-w41inmrl/evidence.json`.

- 4,608 exact oracle comparisons across odd/uneven/thin slabs, 2..4 logical
  devices, eight seeds, overlap/reference and forced/mixed modeled staging.
- 21 failure/admission cases: identity mismatch, duplicate, partial copy/launch,
  cancellation/drop seam, timeout, backend copy/launch API errors, peer unavailable,
  bad shape and readback failure. Readback now poisons the owner and prevents retry.
- Concurrent independent simulations produce their respective CPU references.
- Three child controls exit 70: immediate query error, full two-second stalled
  drain deadline, and an asynchronous queued device-fault/query-error seam.
- Five supervisor assertions pass: success marker, wrong exit, missing marker,
  timeout, and output-limit rejection. Evidence is in
  `docs/evidence/chat10/supervisor-final-xq6ipz26/assertions.json`.

Windows host mode rebuilds source before tests and records the executable digest.
Log preview reads are bounded; hashing uses fixed-size blocks. The real hardware
runner prepares memcheck/initcheck, correctness/stress cases, partial failures,
concurrent processes, five timing repetitions and optional Nsight captures.
`--driver-fault` explicitly adds a real disposable GPU 1 trap as the final test;
it requires logged submission and fail-stop exit 70, and performs no reset/retry.
The real trap has not been compiled or run locally. Windows supervisor assertions
do not validate the POSIX process-group kill implementation on Linux.

Earlier evidence directories record intermediate development candidates. The
`native-final` source/binary digests identify the current tested host coordinator.
Hardware acceptance and a release security review remain pending real GPU results.
