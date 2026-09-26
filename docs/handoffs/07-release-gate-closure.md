# Chat 07: release gate remains FAIL

## 2026-09-26 remediation candidate

Public candidate `57fb542808ccef9060e739459d6f98a26044cb7c` adds an attested
clean-revision release-evidence workflow and provenance guide to the validated
GPU candidate. Hosted run
https://github.com/MannanSaood/gudra/actions/runs/36243147752 passed both CPU
and security jobs. This includes Linux launcher tests, separate-UID private JIT
tests, current RustSec/license/source policy, vendor digests, history secret
scanning, CPU debug/release tests, UI controls, Clippy, formatting, and docs.
The post-GPU delta changes only release workflow, provenance documentation and
manifest generation; runtime, dependencies, GPU policy and tests are identical
to clean pinned GPU candidate `4debea7`.

SEC-004 now has a reviewed Gudra-facade implementation. Root blocking and async
terminals quarantine backend allocations and foreign owners until synchronization,
callback, query, or successful cancellation drain proves completion. Partial
submission errors drain before returning; failed drains and unwinds abort the
worker before owners can drop. Upload wraps raw allocation before the fallible
copy, readback protects its host destination, and zero initialization uses the
retained upload path. An independent review found and caused correction of an
early async-quarantine release; its final pass found no Gudra-public bypass.
Direct applications that separately depend on backend crates are outside Gudra's
supported facade.

The implementation and final device evidence are complete. Clean pinned candidate
`4debea7` passed all 24 commands, including host state-machine tests,
forced-pending cancellation, the CUDA 719 sticky device-fault fail-stop,
five repeated GPU runs, and Compute Sanitizer memcheck/initcheck with zero errors.
Evidence SHA-256 is
`423168e3171311e352eb1ea02017540f24c5d0a8bee3eb64e305c0a6897fade0`.
SEC-001, SEC-004, and final GPU validation are closed. Repository reporting/protection/account/MFA/package
ownership/retention and signed provenance settings remain externally unverified.
The gate therefore remains **FAIL**.

## Current remaining validation

The earlier clean Arch experimental run on 2026-09-26 passed all 22 recorded commands
on `38bb3d1`. It covered the actual private JIT lifecycle, GPU compilation and
Clippy, API docs/doctests, ownership compile-fail controls, deterministic
fail-stop/retention seams, five repeated numerical/readback/budget/cancellation
runs, and Compute Sanitizer memcheck/initcheck with zero errors. Evidence
SHA-256: `864f2cd6b18361229ec175ce4fac12fb936196684d2a4b9f8363abe6fe282821`.
SEC-001 is closed for supported invocation paths. That normal workstation lane
did not enable the deliberate sticky device-fault subprocess. The later clean
pinned `4debea7` disposable run supplied that final evidence and closed SEC-004
and final GPU validation.

The first Arch attempt on 2026-09-25 was bound to clean revision `1ee62e5`
and evidence SHA-256
`4301060bafe0c47b2afd2f5aa828d43cc00db174337c8704ceefaff2a3886baa`.
Rust 1.89.0, CUDA 13.4, and tileiras 13.4 were present, but `nvidia-smi`
exited 9 because it could not communicate with the NVIDIA driver. The runner
failed closed before dependency inspection, compilation, JIT execution, GPU
tests, or Compute Sanitizer. This is prerequisite failure evidence, not a pass.

The second clean Arch attempt on 2026-09-26 reached the RTX 5060, CUDA 13.4,
tileiras, and Compute Sanitizer, then exposed an invalid `cargo -p` selector for
vendored packages. No security or GPU tests ran. Public candidate `38bb3d1`
selects the crates by manifest path and keeps their outputs under the ignored
root target directory. It passed hosted CPU/security CI and requires a fresh
GPU run.

A later clean disposable-worker attempt on `38bb3d1` reached the same RTX 5060,
CUDA 13.4, matching tileiras 13.4, driver 615.71.09, compute capability 12.0,
and Compute Sanitizer. It stopped at an overly narrow preflight requiring exact
CUDA 13.3 and exact Clang 18 package names, before compilation or fault tests.
Evidence SHA-256:
`73a8c74e4bc6c352895fba0e35887f0239e92bfc734eb5c0e7d664ef8ebf6743`.
Candidate `4debea7` accepts CUDA 13.3 and later within major 13, requires exact
`nvcc`/`tileiras` major-minor agreement, and checks Clang/libclang capabilities.
The runtime and sanitizer checks are unchanged.

On a trusted ordinary Arch Linux/NVIDIA workstation, run the non-destructive
candidate lane at an exact clean checkout:

```sh
git fetch origin
git checkout --detach 4debea77627b5e9a6d3d4048036352885c108b0b
test -z "$(git status --porcelain)"
export CUDA_TOOLKIT_PATH=/opt/cuda
bash scripts/verify-gpu.sh
sha256sum target/gpu-candidate-evidence.json
```

This command has now passed and is retained as the reproduction procedure.
`verify-gpu.sh` adds the fault subprocess and may poison a CUDA context, so it
remains limited to an explicitly authorized disposable isolated worker. Release administration remains pending: private
reporting, protected refs and required checks/reviews, maintainer MFA, minimal
package ownership, log/artifact access and retention, native redistribution
rights, clean immutable builds, SBOM, artifact checksums, and signed provenance.

## Historical 2026-09-22 record

Date: 2026-09-22. Private baseline f38c566; this handoff belongs to the local
remediation commit. Public candidate: `401b66526c22a862d8aeaa8ac82a2a1175ac18df`.
See `docs/security/evidence/chat07/source-binding.json` for source binding.

## Blocker matrix and outcomes

| Workstream | Starting evidence | Implementation/result | Environment | Owner/decision | Completion criterion |
|---|---|---|---|---|---|
| SEC-001 | Registry compiler shared transient files | Vendored pinned Apache-2.0 compiler; private Linux directory covers compile and version probe; exclusive input/regular output; cleanup; dependency digest enforcement | Hosted separate-UID tests and final real compiler/GPU lifecycle PASS | Repository owner until delegated | Complete: supported invocation paths verified |
| SEC-004 | Failed drain did not retain outer owners | Dependency terminals retain/quarantine owners; failed completion enforces worker fail-stop | Deterministic seams, cancellation, drop-in-flight, CUDA 719 fault test and sanitizers PASS | Repository owner until delegated | Complete: owner retention or fail-stop verified |
| GPU | Historical results only | Candidate runner records identities, clean tree, commands, exit codes, UI/docs/numerics/budgets/repetitions/sanitizers | Clean pinned Arch/RTX 5060 disposable run: 24/24 PASS | Repository owner until delegated | Complete at immutable candidate `4debea7` |
| Linux/CI | Prior Windows-only evidence | Hardened CPU/security workflow and launcher/installer/vendor/cross-UID tests PASS at public candidate | Hosted Ubuntu disposable runner | Required-check enforcement still owner-controlled | CI executed; external enforcement belongs to release administration |
| Release | No selected license/settings evidence | Owner authorized MIT choice; license installed; compiler remains Apache-2.0. Unsigned manifest/checksum/SBOM tooling and instructions added | Signing/registry/repository settings unavailable; API protection request HTTP 401 | Repository/release owner | Verify reporting, reviews/checks/ref protection, MFA/minimal owners, retention, native rights, signed provenance |
| Review/gate | Recorded FAIL | Independent investigation and source-bound review; technical gates pass, administration remains | Local review + hosted CI + disposable GPU | Repository/release owner | External release protections and provenance verified |

## Published vs private

The user explicitly authorized pushing code to main for a friend's tests, and
explicitly prohibited public audits. A separate worktree at `../gudra-public`
started at the live remote `81d0efb`; only implementation/tests/CI/license/test
instructions were copied. Public commits are ce11aa3, 3947798, e590d0e, ab08927,
401b665. The first two private audit commits ad1c410/5b27b84 and merge f38c566 are
not in their ancestry. No SECURITY.md, docs/security, private findings, evidence,
or Chat 06/07 handoff was published. Never push the private audit branch to origin.
The public workflow differs only by omitting the private inventory-document check;
it verifies vendor identities independently. Repository/package publication stays
false. No package/release, CVE, upstream contact or public finding issue occurred.

## Observed validation

Local Rust 1.89 compiler with Cargo 1.90: 23 CPU tests and one doctest pass in
both modes; UI and docs pass. Clippy/rustfmt 1.90 pass. Installer controls and
locked dependency policy pass. cargo-deny 0.20.2 advisory/license/bans/source
checks pass with database 17af77682cecd2afa72b217ad7c6c30585d5003f; duplicate
warnings unchanged. Gitleaks 8.30.1 current/history scans pass. Vendor provenance:
original registry package checksum 7cbc52ad2ae4e8a4440800673f8ccde58909f25c705d82d4925e025f6c3f3565;
upstream revision 9c11d588a6c4c01daf936c243531580f0ba6e262; unchanged build script
reads header/version only; no native package sources. Prior dependency review
retained; no new registry versions were selected. Own license and vendor notices
are separate. Native CUDA redistribution is not granted.

Hosted run https://github.com/MannanSaood/gudra/actions/runs/35736906616 reports
CPU/security success at 401b665, including Linux launcher, verified audit tool
installation, tamper fixtures, helper unit tests and separate UIDs. Saved JSON
contains job/step conclusions. This does not execute run_tileiras or GPU code.
Local GPU compilation fails before changed backend type-check because CUDA is
missing. WSL access denied; Docker daemon unavailable. No tests skipped as passes.

Independent reviewer found no concrete distinct-UID bypass in the JIT design;
required actual compiler-path tests were then added (still unexecuted). Review
covered input creation, output checks, compiler probe/retry/cache-miss paths,
error cleanup, platform restriction and unchanged persistent-cache assumptions.
An initial generated checksum triggered a secret heuristic; the filename/hash
format was separated into structured fields, with no scanner exclusions.

## Exact next steps

Friend, in a clean checkout of the public candidate with trusted source/toolchain:

```sh
git pull --ff-only
git rev-parse HEAD
python3 scripts/run-private-gpu.py -- python3 scripts/verify-gpu-candidate.py --arch-experimental
```

Review `target/gpu-candidate-evidence.json` privately before sharing. This is the
Arch lane, not a pass for the pinned Ubuntu/CUDA environment. The pinned worker
must run `bash scripts/verify-gpu.sh`. The runner includes
`cargo test --locked -p cutile-compiler --lib private_compilation_lifecycle -- --test-threads=1`.
Never execute imported PR binaries/PTX/cubins. No device faults on the friend's
normal desktop: first establish an explicitly disposable trusted worker.

SEC-004 must be redesigned below dependency terminals: sync_on drops results on
failed sync; execute can drop backend-local owners during error/unwind; future
results may be borrowed. Add deterministic submission/error/panic/registration/
query/bind/drain/drop seams observing real owner and budget destruction, plus
forced-pending cancellation and isolated GPU tests. Current invalidation/leaks
prove neither quiescence nor retention. No local guard was represented as sound.

Repository owner must demonstrate a working private reporting channel, required
reviews/checks, protected release refs/workflows, MFA and minimal package owners,
and actual log access/retention. Do not send credentials/recovery codes. Follow
`docs/security/release-provenance.md` for unsigned evidence and the still-required
protected signing/verification design. The generated SBOM inventories Rust only;
it is not a native/runtime completeness claim. No distributed artifacts exist to
sign or checksum under this prompt.

Run `python3 scripts/check-release-gate.py`: expected exit 1 with only the
release-administration check pending. Gate FAIL is evidence-backed,
not a claim of a confirmed high/critical exploit. There is no accepted remaining
medium-risk waiver and no release authorization.
