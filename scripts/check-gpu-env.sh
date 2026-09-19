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

toolkit_path="${CUDA_TOOLKIT_PATH:-/usr/local/cuda-13.3}"
if [[ -f "$toolkit_path/include/cuda.h" ]]; then
    pass "CUDA headers found under $toolkit_path"
else
    fail "CUDA 13.3 headers missing; set CUDA_TOOLKIT_PATH=/usr/local/cuda-13.3"
fi

nvcc_version="$(nvcc --version 2>/dev/null | sed -n 's/.*release \([0-9][0-9]*\.[0-9][0-9]*\).*/\1/p' | head -n 1)"
if [[ "$nvcc_version" == "13.3" ]]; then
    pass "CUDA Toolkit 13.3 (nvcc)"
else
    fail "CUDA Toolkit 13.3 required (nvcc release: ${nvcc_version:-missing})"
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
    if [[ "$tileiras_version" == "13.3" ]]; then
        pass "tileiras 13.3"
    else
        fail "tileiras must match CUDA 13.3 (reported release: ${tileiras_version:-unparsed})"
    fi
else
    fail "tileiras missing at $tileiras_bin; install the full CUDA 13.3 Toolkit"
fi

if command -v clang-18 >/dev/null 2>&1 && ldconfig -p 2>/dev/null | grep -q 'libclang-18.so'; then
    pass "Clang/libclang 18 available for cuda-bindings bindgen"
else
    fail "clang-18 and libclang-18-dev are required to build cuda-bindings"
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
            pass "NVIDIA driver branch R610+ matches CUDA 13.3"
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
