# Supported configuration and version policy

| Configuration | Admission/support intent | Evidence and release status |
|---|---|---|
| CPU, Windows x86_64 MSVC | Rust 1.89 minimum; default features; native linker | Local 1.89 rustc/rustdoc with Cargo 1.90; stable 1.90 lint tools; exact pinned installation incomplete |
| CPU, Linux x86_64 | Rust/Cargo/rustfmt/Clippy 1.89.0; hosted Ubuntu 24.04 | Historical hosted pass; current final immutable candidate must rerun |
| GPU, Linux NVIDIA | Rust 1.89.0, patched cuTile 0.3.1, full CUDA 13.3+ within 13.x, matching nvcc/tileiras, R610+ driver, CC 8.0+, Clang/libclang 18+ | Admission range, not a tested cross-product; current candidate unverified |
| Recorded single GPU | Arch Linux, RTX 5060 Laptop, CC 12.0, CUDA/tileiras 13.4, driver 615.71.09, Clang/libclang 22 | Historical base and manifest-matched Chat 09 evidence; not final release attestation |
| GPU Windows/macOS, CUDA 12 or 14, older drivers/CC | Not supported | Private compiler override is Linux-only; do not infer support from registry crates |
| Multi-GPU | Prototype only, excluded from candidate | Two-GPU execution/sanitizers/transport/fault validation pending |

GPU numerical fixtures use `1e-5 + 1e-5*abs(reference)`, and nonfinite
classifications are checked separately; this is not a scientific error bound.
No GPU hardware/toolkit is available on the current local release worker.

Use SemVer: first reviewed candidate `0.1.0-rc.1`, followed by incrementing rc
numbers for changed candidates. Stable `0.1.0` requires all gates plus explicit
publication authorization. Before 1.0, incompatible API, numerical convention,
ownership or configuration changes require a minor version; compatible fixes
use patch versions. Treat an MSRV/CUDA/driver increase as a compatibility change
and announce it in release notes. Never reuse a tag/version for different bytes.

Rust 1.89 is the MSRV and exact 1.89.0 is the reproducibility baseline. Test
minimum and current stable on CPU; final acceptance requires the complete pinned
toolchain, not a mixed-toolchain workaround. Keep the lockfile and vendor manifest
with every source candidate. Upgrading a backend or native-tool requirement needs
fresh capability/license/security review and a new exact-source GPU run. CUDA
minor releases within the admission range require measured validation before a
specific lane is advertised as verified. Do not promise untested range members.
