# Deliberately forbidden in-place update

`in_place_jacobi.cu` reads and writes the same global allocation during one
five-point stencil. A cell's write conflicts with its neighbors' reads, including
across blocks. Block barriers cannot establish a grid-wide snapshot. It illustrates
the access pattern excluded by Gudra's safe ownership API; it is not a Gudra backend.

Build and run on a CUDA C++ host:

```sh
nvcc -std=c++17 -O2 -lineinfo fixtures/in_place_jacobi.cu -o target/in_place_jacobi
./target/in_place_jacobi
compute-sanitizer --tool memcheck --error-exitcode 99 ./target/in_place_jacobi
compute-sanitizer --tool racecheck --error-exitcode 99 ./target/in_place_jacobi
```

Each of 20 trials restores the same initialized input and compares to a separate
CPU update. Counts can be identical, different, or zero across runs and devices.
The program intentionally does not assert that a mismatch must occur; exit 0
only means the demonstration ran without a reported CUDA API failure.

NVIDIA's current [Compute Sanitizer Racecheck documentation](https://docs.nvidia.com/compute-sanitizer/ComputeSanitizer/index.html#what-is-racecheck)
describes detection of on-chip shared-memory hazards (accessed 2026-09-19).
Racecheck does not generally detect global-memory races such as this stencil.
A clean report cannot establish its correctness. Memcheck checks invalid memory
accesses; initialized, in-bounds accesses can still race.

This is an intentional numerical correctness counterexample, not a CVE. A CVE
claim would require a distinct security impact and an appropriate coordinated
process. The local host lacks CUDA; compilation and observations are pending.
