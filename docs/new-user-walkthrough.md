# Performed new-user walkthrough

Performed 2026-10-12 by the maintainer's coding agent on Windows x86_64.
This is a fresh **source/build environment**, not an independent reviewer or a
new OS installation. A disposable directory contained Cargo.toml/lock,
rust-toolchain.toml, src, tests, examples, vendor and the UI runner. No checkout
Git state, target artifacts or private machine configuration was copied.
CARGO_TARGET_DIR pointed to an initially absent target inside that copy. The
copy was removed after checks. Installed tools, native linker and cached
registry dependencies were reused; offline results do not verify first-time
downloads, rustup installation or a public clone of this unpublished delta.

## Starting diagnostics

Plain `cargo --version` fails: the pinned 1.89.0 toolchain lacks Cargo here.
The documented recovery uses installed Cargo 1.90.0 with explicit Rust/rustdoc
1.89.0 paths, without changing toolchain files or lock resolution. No CUDA tools
are available on PATH. See [captured diagnostics](evidence/chat11/diagnostics.json)
and [troubleshooting](troubleshooting.md). A complete pinned setup is still a
prerequisite for the normal commands; mixed-toolchain checks are labelled as such.

## Steps actually performed

The [walkthrough runner](../scripts/run-doc-walkthrough.py) creates the clean
copy, hashes its inputs and runs offline checks. Within the checkout, on this
Windows installation:

```powershell
python scripts/run-doc-walkthrough.py --cargo "$env:USERPROFILE\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe" --rustc "$env:USERPROFILE\.rustup\toolchains\1.89.0-x86_64-pc-windows-msvc\bin\rustc.exe" --rustdoc "$env:USERPROFILE\.rustup\toolchains\1.89.0-x86_64-pc-windows-msvc\bin\rustdoc.exe" --output docs/evidence/chat11
```

On a complete installation use `python3 scripts/run-doc-walkthrough.py --output
<local-evidence-directory>`; fetch locked dependencies first if needed. Avoid
committing unreviewed output. The output argument selects logs, not source input.

| Check | Observed result |
|---|---|
| Locked offline build | PASS, fresh target |
| CPU debug tests | PASS: 28 unit/integration tests + 1 doctest, none ignored |
| CPU release tests | PASS: same counts |
| Minimal cpu_step | PASS: old center=4, new center=3.5 |
| Release curvature | PASS: all 323 cells equal 6 |
| Release Poisson residual | PASS: after 500 steps, max residual=0.051231384 |
| CPU ownership runner | PASS: positive control; Jacobi and Laplacian alias cases rejected with E0502 |
| Public CPU API docs | PASS |
| GPU execution / two-GPU run | NOT RUN on this host |

[Structured exits/commands](evidence/chat11/results.json) and adjacent redacted
logs are the primary evidence. [Source manifest](evidence/chat11/source-manifest.json)
SHA-256: `78acea63e2e665cce61852e31b0c60ab5abde0fdb589dadfdc4844d8530fb89b`.
The runner was repeated after correcting an exact-float Clippy complaint in the
minimal example. The final logs/manifest describe the corrected source.
An initial OS temporary-directory attempt hit local sandbox permissions before
building; scratch was moved under the checked checkout target directory.
No dependency or GPU safety implementation was changed by this work.

Additional final-checkout verification: stable Rust/Cargo 1.90 formatting and
all-target CPU Clippy with denied warnings passed. Direct rustdoc testing of
README.md with the matching stable CPU library passed both the runnable snippet
and the compile-fail snippet. Compile-fail rustdoc alone can accept any failure;
the separate UI runner verifies the intended E0502 diagnostic and positive control.
The [documentation QA record](evidence/chat11/documentation-qa.json) records
link/template validation and scoped privacy inspection. Logs replace local paths
with placeholders. This is not a new full repository/history security audit.

## Read-back self-review and acceptance limit

The README lets the reader identify the protected failure: writing one grid
while neighbors still need its old values changes Jacobi and can race on a GPU.
Separate storage/borrows and owned pending buffers exclude the supported alias
misuse; halo freshness and convergence remain the application's responsibility.
This is the author's self-review, not proof that a target user understood it.
The [target-user interview](community/feedback.md) and actual cluster setup trial
remain open. Do not report community acceptance until a user answers and their
results are recorded. Broader Rust/CUDA prerequisites are documented target
requirements backed by revision-scoped returned GPU results, not freshly tested
on every advertised platform.
