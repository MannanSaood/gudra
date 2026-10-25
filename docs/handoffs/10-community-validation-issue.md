## Help wanted

Published: https://github.com/MannanSaood/gudra/issues/3 (2026-10-12).

We are looking for a contributor with at least two local NVIDIA GPUs to validate an experimental one-dimensional slab halo-exchange prototype. Development is moving on to documentation/community work while this hardware validation remains an open follow-up.

## Current status

The standalone C++/CUDA worker implements one-cell halos, checked epoch and buffer-generation identities, directed P2P capability checks, pinned-host staging fallback, an overlap-disabled reference mode, and interior/exchange overlap.

Local host-backend checks passed 4,608 CPU-reference generation comparisons, 21 failure/admission cases, concurrent simulations, and cleanup/supervisor checks. The final scoped source security review found no confirmed vulnerabilities.

CUDA compilation, two-GPU numerical correctness, actual P2P/staging behavior, Compute Sanitizer checks, driver-fault disposal, and real overlap profiling are still unverified. Host tests provide no GPU performance evidence.

## Source and runbook

The prototype is not yet on this repository's default branch. Please comment here to coordinate with the maintainer and obtain the source-only package `gudra-halo-native-20261003.zip`.

Package SHA-256: `11f0649749cdf81668aa1d4f104f75d5f49a38bbc9f08ce2cf2609d7bdf2ecd3`.

The package includes the native README/runbook, protocol documentation, source manifest, and local test evidence. After extracting it:

```bash
cd gudra-halo-candidate/prototypes/halo-native
python3 run_checks.py --profile
```

Requirements: trusted Linux host, at least two visible NVIDIA GPUs, compatible CUDA >=12 toolkit/driver, g++, Python 3.9+, Compute Sanitizer, and Nsight Systems. Select the intended pair using `CUDA_VISIBLE_DEVICES`; leave `CUDA_LAUNCH_BLOCKING` unset for overlap measurements. No tools are installed by the runner. Lack of P2P support is useful evidence: staging should work, and P2P coverage must be reported separately.

Use a trusted isolated worker. The default command does not run the optional real sticky GPU-fault test; coordinate separately before selecting the runbook's disposable-worker fault option.

## Evidence requested

Return the complete generated `results/halo-*` directory, including `evidence.json`, command logs, source/binary digests, GPU UUIDs and directed peer capabilities, driver/toolchain details, sanitizer outputs, and Nsight captures. Inspect logs and paths for private information before sharing; coordinate a transfer if evidence exceeds GitHub attachment limits.

Please report compilation failures, wrong results, stale/mismatched halo acceptance, partial-failure continuation, timeout/cleanup problems, or missing expected overlap. Include the exact command and evidence, even for a blocked run. Results should compare reference and overlap modes and include communication/synchronization costs.

## Tracking disposition

Two-GPU availability is no longer a prerequisite for proceeding to the next development stage. This issue tracks deferred hardware validation; it does not declare native GPU acceptance or authorize a production release or performance claim. The tested source must receive another security review before production release.
