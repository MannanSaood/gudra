# RTX 5070 / Arch Linux: Chat 08 validation

This is an unvalidated GPU implementation candidate. Send back failures as well
as successes. The ZIP contains source, vendored changes, tests and instructions;
it contains no CUDA binaries, credentials, build outputs or Git history.

## 1. Prepare a fresh directory

Unzip the source bundle into a new directory and enter `gudra-chat08`. Do not
unzip over an existing working tree. Compare the ZIP SHA-256 against the supplied
`.sha256` file (`sha256sum gudra-chat08-source.zip`). The bundled
`BUNDLE-MANIFEST.json` records every source file's hash and the original base
revision; this candidate includes changes beyond that base.

Use the repository setup in `docs/setup.md`: Rust 1.89.0 with rustfmt/clippy,
full CUDA toolkit with `nvcc`, `tileiras`, and `compute-sanitizer`, compatible
NVIDIA driver, Python 3, Git, Clang/libclang. The repository's reviewed lane
specifies CUDA 13.3+ within 13.x and driver R610+. If Arch has a different stack,
record its exact versions; do not represent the experimental lane as pinned.
Do not reinstall or change a functioning driver merely to run this candidate.

For an Arch CUDA installation under `/opt/cuda` (adjust to the actual install):

```bash
export CUDA_TOOLKIT_PATH=/opt/cuda
export PATH="$CUDA_TOOLKIT_PATH/bin:$PATH"
export LIBCLANG_PATH="$(llvm-config --libdir)"
rustup toolchain install 1.89.0 --profile minimal --component rustfmt --component clippy
nvidia-smi
nvcc --version
tileiras --version
compute-sanitizer --version
```

Missing tools or unsupported Tile IR must be reported, not silently bypassed.
The run executes the trusted repository code and its vendored compiler.

## 2. Record this source snapshot locally

The evidence runner requires a clean Git tree. The ZIP has no `.git` directory.
These commands create a local-only validation revision:

```bash
git init
git add .
git -c user.name='GPU Validation' -c user.email='validation@localhost' commit -m 'Chat 08 source bundle for GPU validation'
git status --short
```

Do not add generated logs to this revision. Logs go under ignored `target/`.
No remote, push or release is required.

## 3. Run the normal Arch lane

```bash
mkdir -p target
set -o pipefail
python3 scripts/run-private-gpu.py -- python3 scripts/verify-gpu-candidate.py --arch-experimental 2>&1 | tee target/chat08-gpu.log
```

This records hardware/toolchain versions, builds actual GPU APIs, runs Clippy,
GPU docs and compile-fail controls, tests the library and GPU suite five times,
runs backend drop coverage, checks both test executables with memcheck and
initcheck, and finally runs the Poisson/timing example in release mode.

New tests compare 125 steps across several shapes against the CPU oracle. The
pending-drop test must actually see Pending; it fails rather than silently
claiming cancellation coverage. The runner disables the short spin budget to
help exercise pending completion. Boundaries include NaN corners, which must
stay outside the interior stencil and preserve their stored bits.

No numbers have been measured yet. The final example prints direct blocking and
owned async median wall times for five trials of 125 steps each, excluding
upload/readback and warmup/JIT. Both paths use the same scalar-tile kernel.
Expect this initial kernel to have poor throughput; do not infer a speedup from
a single run or compare it directly to the old interior-only single-step kernel.

If the runner stops, send the log/JSON as-is. Do not edit tests to skip failures.

## 4. Optional disposable fault lane

The normal command intentionally omits the backend's deliberate illegal-address
fault test. That existing release requirement is still outstanding for the new
candidate. Only on a disposable GPU worker/session where context loss is okay:

```bash
python3 scripts/run-private-gpu.py -- python3 scripts/verify-gpu-candidate.py --arch-experimental --disposable-fault-tests 2>&1 | tee target/chat08-fault.log
```

Do not run this lane while the GPU is doing important work. The normal lane can
be returned first; mark the fault lane NOT RUN if unsuitable on the friend's PC.

## 5. Return evidence

Send back:

- `target/gpu-candidate-evidence.json` (created progressively, even on failure)
- `target/chat08-gpu.log` and fault log if that lane was run
- `BUNDLE-MANIFEST.json` and the original ZIP SHA-256
- any extra error text if execution stopped before writing JSON

The JSON contains exact local revision, versions, command exit codes and output.
Review logs before sharing for machine-specific paths/usernames. A new kernel
requires these new results; earlier release evidence is not a substitute.
