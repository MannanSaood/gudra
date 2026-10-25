# ADR 008c: Defer CUDA Graph capture and replay

Status: accepted; graph optimization deferred.

The vendored backend has graph support, but the existing direct terminal and
cancellation drain proof are the reviewed facade path. This change does not add
a graph API, graph cache, capture-time allocation, or persistent raw addresses.
No measured graph benefit is available on the current machine.

A future graph executable must own both grids, RHS and session for its entire
lifetime, including replay and cancellation. Replay must exclude mutable access
until completion, pin shapes/strides/dtype/device and parameters to the captured
contract, and reject changed metadata. Allocation addresses must remain stable;
no cache keyed only by shape can substitute arbitrary owners. Warm up JIT before
capture; exclude allocations/readback from capture unless independently proved.

A two-step graph can return to the same current/next orientation. Odd replays
need a distinct executable or explicit parity tracking; replaying a one-step
capture against unchanged baked-in pointers does not automatically swap roles.
Graph cloning or simultaneous launches on another stream must not create mutable
aliases. Capture errors, executable destruction, failed drain and context faults
need dedicated tests and security review before enabling replay. These conditions
preserve the same ownership and shape assumptions; graph support alone does not.
