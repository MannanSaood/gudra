# Troubleshooting

Start with revision, `rustc --version`, `cargo --version`, `rustup show active-toolchain`
and the exact command. See [reporting](community/reporting.md) before sharing logs.

| Symptom | Diagnosis and action |
|---|---|
| cargo binary is not applicable to 1.89.0 | Incomplete rustup toolchain; install the Cargo component or reinstall 1.89.0. The measured local workaround is below; it is not pinned Cargo validation. |
| linker/link.exe/cc missing | Install platform native build tools (MSVC C++ tools on Windows; C compiler/linker on Linux). |
| offline build cannot find a package | First run `cargo fetch --locked` with network access; then retry. Offline assumes cached dependencies. |
| dependency build cannot find CUDA | CPU default builds do not need CUDA. GPU builds require Linux and the full reviewed toolkit; run `bash scripts/check-gpu-env.sh`. |
| tileiras absent or mismatches nvcc | Select one full toolkit via CUDA_TOOLKIT_PATH and PATH; do not mix installations. |
| libclang or stddef.h missing | Install Clang/libclang and development headers; set LIBCLANG_PATH to the actual library directory. |
| GPU UI runner says BLOCKED | Fix the positive library build first. No compile-fail result was counted as passing. |
| Wrong length, same area but different shape | Use Shape2D(height,width); halo and compact output have different extents. Check rows, columns and stride, not only element count. |
| WrongSession | Create all buffers with the same Gpu instance; equal device ordinals do not imply equal sessions. |
| InvalidBuffer or execution error | Uncertain output cannot be read/reused. Do not retry a device fault in the same session. Worker abort is the intended failed-completion policy. |
| NaN/infinity or unstable residual | Inspect coefficients, data, halo and discretization. Ownership does not guarantee convergence; do not enlarge tolerance to hide a failure. |
| First launch slow / GPU slower for tiny grids | JIT and launch overhead matter. Compare warmed and end-to-end scopes separately; see benchmarks. |
| Async cancellation seems stuck | Drop may drain the ordered stream synchronously. It does not preempt a kernel or promise bounded latency. |
| Native halo run lacks P2P | Report directed capability and test staging separately; do not claim P2P coverage. |

## Measured local toolchain workaround

The Chat 11 Windows host has a Rust 1.89 compiler/rustdoc but no pinned Cargo.
The clean-source CPU walkthrough uses installed stable Cargo 1.90 with explicit
1.89 compiler/rustdoc. In PowerShell, within the checkout:

```powershell
$env:RUSTC = "$env:USERPROFILE\.rustup\toolchains\1.89.0-x86_64-pc-windows-msvc\bin\rustc.exe"
$env:RUSTDOC = "$env:USERPROFILE\.rustup\toolchains\1.89.0-x86_64-pc-windows-msvc\bin\rustdoc.exe"
$env:CARGO = "$env:USERPROFILE\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe"
cargo +stable test --locked --offline
cargo +stable run --locked --offline --example cpu_step
python scripts/check-ui.py
Remove-Item Env:RUSTC, Env:RUSTDOC, Env:CARGO
```

Use `python` on Windows if `python3` is unavailable. On a full pinned installation,
the normal setup commands need no workaround. Record the actual compiler and
Cargo separately; do not call mixed-toolchain results fully pinned.
