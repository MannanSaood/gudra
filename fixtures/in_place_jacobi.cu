// Intentionally incorrect global-memory stencil. Educational negative control.
// A mismatch is informative; matching runs do NOT establish correctness.
#include <cuda_runtime.h>
#include <algorithm>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <vector>

static void check(cudaError_t result, const char* operation) {
    if (result != cudaSuccess) {
        std::fprintf(stderr, "%s: %s\n", operation, cudaGetErrorString(result));
        std::exit(2);
    }
}

__global__ void forbidden_in_place(float* u, const float* rhs, int h, int w) {
    const int r = blockIdx.y * blockDim.y + threadIdx.y;
    const int c = blockIdx.x * blockDim.x + threadIdx.x;
    if (r >= h || c >= w) return;
    const int stride = w + 2;
    const int i = (r + 1) * stride + c + 1;
    // Neighbors read u[i] while this thread overwrites it. Different blocks
    // have no ordering; __syncthreads() would not fix cross-block accesses.
    const float candidate = 0.25f *
        (u[i - stride] + u[i + stride] + u[i + 1] + u[i - 1] - 0.25f * rhs[r * w + c]);
    u[i] = 0.5f * u[i] + 0.5f * candidate;
}

int main() {
    constexpr int h = 257, w = 263, stride = w + 2, trials = 20;
    std::vector<float> initial((h + 2) * stride), rhs(h * w), expected(h * w);
    for (int r = 0; r < h + 2; ++r)
        for (int c = 0; c < w + 2; ++c)
            initial[r * stride + c] = float((r * 17 + c * 31 + r * c) % 97 - 48) / 8.0f;
    for (int i = 0; i < h * w; ++i) rhs[i] = float(i % 13 - 6) / 4.0f;
    for (int r = 0; r < h; ++r) {
        for (int c = 0; c < w; ++c) {
            const int i = (r + 1) * stride + c + 1;
            expected[r * w + c] = 0.5f * initial[i] + 0.5f * 0.25f *
                (initial[i - stride] + initial[i + stride] + initial[i + 1] + initial[i - 1] - 0.25f * rhs[r * w + c]);
        }
    }
    float *device_u = nullptr, *device_rhs = nullptr;
    const size_t bytes = initial.size() * sizeof(float);
    check(cudaMalloc(reinterpret_cast<void**>(&device_u), bytes), "allocate source/output");
    check(cudaMalloc(reinterpret_cast<void**>(&device_rhs), rhs.size() * sizeof(float)), "allocate RHS");
    check(cudaMemcpy(device_rhs, rhs.data(), rhs.size() * sizeof(float), cudaMemcpyHostToDevice), "upload RHS");
    std::vector<float> actual(initial.size());
    for (int trial = 0; trial < trials; ++trial) {
        check(cudaMemcpy(device_u, initial.data(), bytes, cudaMemcpyHostToDevice), "reset trial");
        forbidden_in_place<<<dim3((w + 15) / 16, (h + 15) / 16), dim3(16, 16)>>>(device_u, device_rhs, h, w);
        check(cudaGetLastError(), "launch");
        check(cudaDeviceSynchronize(), "complete");
        check(cudaMemcpy(actual.data(), device_u, bytes, cudaMemcpyDeviceToHost), "readback");
        int mismatches = 0;
        float max_error = 0.0f;
        for (int r = 0; r < h; ++r) {
            for (int c = 0; c < w; ++c) {
                const float a = actual[(r + 1) * stride + c + 1];
                const float e = expected[r * w + c];
                const float error = std::abs(a - e);
                if (!std::isfinite(a) || error > 1e-5f + 1e-5f * std::abs(e)) ++mismatches;
                max_error = std::max(max_error, error);
            }
        }
        std::printf("trial=%d mismatches=%d max_abs_error=%g\n", trial, mismatches, max_error);
    }
    check(cudaFree(device_rhs), "free RHS");
    check(cudaFree(device_u), "free source/output");
    std::puts("Observations only: no deterministic race reproduction or safety verdict.");
    return 0;
}
