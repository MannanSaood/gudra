ARG RUST_IMAGE=rust:1.89.0-slim-bookworm@sha256:294917190b5a3fed18d3303213943f40ec9b644b3cd6e5cdc8cd029334a770b3
ARG CUDA_IMAGE=nvidia/cuda:13.3.0-devel-ubuntu24.04@sha256:69e9e39eb8fe2cda271654a0f5eac2f1bb946b2fb9c460eb19c7c3c155f4e64e

FROM ${RUST_IMAGE} AS rust
FROM ${CUDA_IMAGE}

COPY --from=rust /usr/local/cargo /usr/local/cargo
COPY --from=rust /usr/local/rustup /usr/local/rustup

ENV CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/cargo/bin:/usr/local/cuda-13.3/bin:${PATH} \
    CUDA_TOOLKIT_PATH=/usr/local/cuda-13.3 \
    LIBCLANG_PATH=/usr/lib/llvm-18/lib

# cuda-bindings 0.3.1 runs bindgen at build time. The CUDA devel image has
# the CUDA headers/compiler but not the Clang resource headers bindgen needs.
RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        clang-18=1:18.1.3-1ubuntu1 \
        libclang-18-dev=1:18.1.3-1ubuntu1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
CMD ["bash"]
