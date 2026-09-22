# GPU candidate testing

This is an experimental candidate, not release approval. Do not use production
credentials/data or download prebuilt test binaries. Build locally from the exact
reviewed commit with a clean working tree. No device-fault injection is included.

For the pinned Linux/CUDA environment:

```sh
bash scripts/verify-gpu.sh
```

For the separately identified Arch environment (records actual installed CUDA,
compiler, driver and GPU; does not count as the pinned environment passing):

```sh
python3 scripts/run-private-gpu.py -- python3 scripts/verify-gpu-candidate.py --arch-experimental
```

Both execute the same normal-path numerical, ownership UI, docs, repetition and
sanitizer suite, without changing tolerances. Failures stop the run. Inspect and
privately share target/gpu-candidate-evidence.json, which includes revision,
dirty status, versions, commands, outputs and exit codes. Paths are partially
redacted; manually review diagnostics before sharing. Remove working logs within
7 days after preserving the reviewed evidence privately.

Cross-user tests must run separately in a disposable Linux VM, with two existing
non-root test accounts and no unrelated workloads. They require root only to
select the two UIDs; do not run them on the desktop used for normal GPU work:

```sh
sudo python3 scripts/test-private-jit-linux.py --disposable --victim jit-victim --attacker jit-attacker
```

Process abort/SIGKILL can leave private compiler directories. After verifying all
compiler processes have exited, the operator may remove only the abandoned
cutile-private directories owned by that worker. Do not share a worker UID with
untrusted processes. The normal suite does not establish device-fault recovery,
forced-pending cancellation, cross-user isolation, or release readiness.
