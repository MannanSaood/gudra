#!/usr/bin/env bash
set -uo pipefail

failures=0

pass() {
    printf 'PASS  %s\n' "$1"
}

fail() {
    printf 'FAIL  %s\n' "$1" >&2
    failures=$((failures + 1))
}

cuda_version_supported() {
    local version="$1" major minor
    [[ "$version" =~ ^([0-9]+)\.([0-9]+)$ ]] || return 1
    major="${BASH_REMATCH[1]}"
    minor="${BASH_REMATCH[2]}"
    (( 10#$major == 13 && 10#$minor >= 3 ))
}

if [[ "${1:-}" == "--policy-self-test" ]]; then
    for version in 13.3 13.4 13.99; do
        cuda_version_supported "$version" || fail "policy rejected supported CUDA $version"
    done
    for version in 13.2 12.9 14.0 invalid; do
        if cuda_version_supported "$version"; then
            fail "policy accepted unreviewed CUDA $version"
        fi
    done
    if (( failures > 0 )); then
        exit 1
    fi
    pass "CUDA version policy accepts CUDA 13.3+ within major version 13"
    exit 0
fi

if [[ "$(uname -s 2>/dev/null)" == "Linux" ]]; then
    pass "Linux host ($(uname -m))"
else
    fail "cuTile Rust is supported on Linux; use the pinned devcontainer on another host OS"
fi

rust_version="$(rustc --version 2>/dev/null | awk '{print $2}')"
if [[ "$rust_version" == "1.89.0" ]]; then
    pass "rustc 1.89.0"
else
    fail "rustc 1.89.0 required by rust-toolchain.toml (found: ${rust_version:-missing})"
fi

toolkit_path="${CUDA_TOOLKIT_PATH:-/usr/local/cuda}"
if [[ -f "$toolkit_path/include/cuda.h" ]]; then
    pass "CUDA headers found under $toolkit_path"
else
    fail "CUDA headers missing; set CUDA_TOOLKIT_PATH to CUDA 13.3 or newer within major version 13"
fi

nvcc_version="$(nvcc --version 2>/dev/null | sed -n 's/.*release \([0-9][0-9]*\.[0-9][0-9]*\).*/\1/p' | head -n 1)"
if cuda_version_supported "$nvcc_version"; then
    pass "supported CUDA Toolkit $nvcc_version (nvcc)"
else
    fail "CUDA Toolkit 13.3+ within major version 13 required (nvcc release: ${nvcc_version:-missing})"
fi

tileiras_bin="${CUTILE_TILEIRAS_PATH:-$toolkit_path/bin/tileiras}"
if [[ -x "$tileiras_bin" ]]; then
    tileiras_output="$($tileiras_bin --version 2>&1)"
    tileiras_version="$(
        printf '%s\n' "$tileiras_output" \
            | sed -nE 's/.*release ([0-9]+\.[0-9]+).*/\1/p' \
            | head -n 1
    )"
    if [[ -z "$tileiras_version" ]]; then
        tileiras_version="$(
            printf '%s\n' "$tileiras_output" \
                | sed -nE 's/.*V([0-9]+\.[0-9]+)(\.[0-9]+)?.*/\1/p' \
                | head -n 1
        )"
    fi
    if cuda_version_supported "$tileiras_version" && [[ "$tileiras_version" == "$nvcc_version" ]]; then
        pass "tileiras $tileiras_version matches nvcc"
    else
        fail "tileiras must be CUDA 13.3+ and match nvcc $nvcc_version (reported: ${tileiras_version:-unparsed})"
    fi
else
    fail "tileiras missing at $tileiras_bin; install the full reviewed CUDA toolkit"
fi

clang_bin="${CLANG:-}"
if [[ -z "$clang_bin" ]]; then
    clang_bin="$(command -v clang-18 2>/dev/null || command -v clang 2>/dev/null || true)"
fi
clang_version="$($clang_bin --version 2>/dev/null | sed -nE '1s/.*version ([0-9]+).*/\1/p')"
libclang_available=0
libclang_location=""
libclang_candidates=()
if [[ -n "${LIBCLANG_PATH:-}" ]]; then
    libclang_candidates+=("${LIBCLANG_PATH%/}")
fi
if command -v llvm-config >/dev/null 2>&1; then
    libclang_candidates+=("$(llvm-config --libdir 2>/dev/null || true)")
fi
clang_resource_dir="$($clang_bin --print-resource-dir 2>/dev/null || true)"
if [[ -n "$clang_resource_dir" ]]; then
    libclang_candidates+=("$(dirname "$(dirname "$clang_resource_dir")")")
fi
libclang_candidates+=(/usr/lib /usr/lib64 /usr/local/lib /usr/local/lib64)
for candidate in "${libclang_candidates[@]}"; do
    if [[ -n "$candidate" ]] && compgen -G "$candidate/libclang.so*" >/dev/null; then
        libclang_available=1
        libclang_location="$candidate"
        break
    fi
done
if (( libclang_available == 0 )) && ldconfig -p 2>/dev/null | grep -Eq 'libclang[^ ]*\.so'; then
    libclang_available=1
    libclang_location="system linker cache"
fi
if [[ "$clang_version" =~ ^[0-9]+$ ]] && (( clang_version >= 18 )) && (( libclang_available == 1 )); then
    pass "Clang $clang_version and libclang at $libclang_location available for cuda-bindings bindgen"
else
    fail "Clang 18+ and discoverable libclang are required (clang major: ${clang_version:-missing}; set LIBCLANG_PATH if libclang is installed outside standard LLVM locations)"
fi

if command -v nvidia-smi >/dev/null 2>&1; then
    gpu_rows="$(nvidia-smi --query-gpu=driver_version,compute_cap,name --format=csv,noheader 2>/dev/null)"
    if [[ -z "$gpu_rows" ]]; then
        fail "nvidia-smi found but returned no GPUs"
    else
        supported_gpu=0
        supported_driver=0
        while IFS=',' read -r driver compute_cap name; do
            driver="${driver//[[:space:]]/}"
            compute_cap="${compute_cap//[[:space:]]/}"
            name="${name# }"
            printf 'INFO  GPU=%s driver=%s compute_capability=%s\n' "$name" "$driver" "$compute_cap"
            if awk -v capability="$compute_cap" 'BEGIN { exit !(capability >= 8.0) }'; then
                supported_gpu=1
            fi
            if [[ "${driver%%.*}" =~ ^[0-9]+$ ]] && (( ${driver%%.*} >= 610 )); then
                supported_driver=1
            fi
        done <<< "$gpu_rows"
        if (( supported_gpu == 1 )); then
            pass "at least one NVIDIA GPU has compute capability 8.0+"
        else
            fail "an NVIDIA GPU with compute capability 8.0+ is required"
        fi
        if (( supported_driver == 1 )); then
            pass "NVIDIA driver meets the R610 floor; runtime tests verify toolkit compatibility"
        else
            fail "NVIDIA driver R610+ is required by this pinned validation lane"
        fi
    fi
else
    fail "nvidia-smi missing; install an NVIDIA driver and ensure the GPU is accessible"
fi

if (( failures > 0 )); then
    printf '\nGPU environment is NOT ready (%d failed check(s)).\n' "$failures" >&2
    printf 'See docs/setup.md for setup instructions.\n' >&2
    exit 2
fi

printf '\nGPU environment is ready.\n'
