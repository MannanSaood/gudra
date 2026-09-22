# Compiler source override

cutile-compiler 0.3.1 is copied from its Cargo registry package, upstream revision
9c11d588a6c4c01daf936c243531580f0ba6e262. It remains Apache-2.0, not MIT.
All original source copyright/SPDX notices are retained. Local modifications
are src/private_temp.rs and src/cuda_tile_runtime_utils.rs: compilation and
version probes use root-anchored private Linux temporary directories, and
failed compilations no longer retain bytecode. Other platforms fail explicitly.

The build script is unchanged: it reads CUDA header/version information and
emits an advisory build warning; it does not download or compile native code.
This package contains no native object/library sources. It still depends on the
locked CUDA/native graph. Proprietary CUDA redistribution is not authorized by
this repository's license. Do not bundle CUDA toolkit/driver files in releases.

vendor/checksums.json binds every copied/modified file (LF-normalized SHA-256).
Changing the override requires reviewing source, license and build capabilities.
Private directories are removed on normal/error/unwind exits. SIGKILL, process
abort and machine loss cannot run destructors: remaining directories stay private
and require operator cleanup after confirming the compiler child has exited.
The compiler installation, same-UID processes and root are trusted. Persistent
cache opt-in still requires separately trusted storage. TMPDIR is deliberately
ignored by this compiler override; /tmp must be root-owned and sticky if writable.
