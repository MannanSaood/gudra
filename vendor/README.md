# Reviewed source overrides

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

## GPU completion dependencies

`cuda-async` 0.3.1 (registry checksum
`a6d51fe2c9b1a89f4ac760d72a197761789f4e60db1d07d3f4d48de65a78adc9`) and
`cutile` 0.3.1 (registry checksum
`422c8fb32ac648113eb032a6d818f09b0862812224bd4a0ed563fc174f6f5264`) are copied
from their Cargo registry packages at upstream revision
`9c11d588a6c4c01daf936c243531580f0ba6e262`. Both remain Apache-2.0. Neither
package has a build script or contains native source files.

The `cuda-async` override adds a fail-stop completion guard to the root
`DeviceOp` blocking and async terminals used by Gudra. A terminal returns only
after stream synchronization proves completion.
While completion is uncertain, backend allocation release and foreign allocation
liveness owners are quarantined on the executing thread. Failed synchronization,
unwind, or a failed cancellation drain
aborts the worker before retained owners can be dropped or reused. CPU subprocess
tests cover these state transitions; final real-device fault injection remains a
release requirement.

The `cutile` override protects the consuming device-to-host transfer's partially
initialized host vector with the same terminal guard. Gudra separately retains
the source device tensor through that terminal. These overrides do not implement
driver recovery or permit same-process retry after a device fault.

Gudra does not expose `cuda_async::simt`, raw device operations, CUDA graphs, or
cuTile tensor constructors as public APIs. The reviewed support boundary is
Gudra's safe GPU facade; applications that declare these implementation crates
as their own direct dependencies must review those additional APIs separately.
