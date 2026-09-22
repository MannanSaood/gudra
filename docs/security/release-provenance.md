# Release provenance and verification (not executed)

Publication remains disabled. Build only after all gate checks close, from an
approved immutable commit in a clean isolated worker without unrelated data.
Never promote PR-produced binaries, cubins, PTX or benchmark artifacts. Verify
source review, required checks and protected tag/environment approval first.

Run cargo build --locked --release with the approved features. Record OS image,
rustc -Vv, cargo -V, CUDA toolkit, tileiras, loaded native libraries, driver/GPU
and every native tool digest. Supply only the intended artifacts to:

```sh
python3 scripts/release-manifest.py --artifact target/release/APP --tool /path/to/rustc --tool /path/to/tileiras --output target/release-evidence
sha256sum -c target/release-evidence/SHA256SUMS
```

The helper emits a CycloneDX Rust dependency inventory, checksums and unsigned
manifest binding source revision, lockfile, vendor source manifest, tools and
artifact digests. It does not include all OS/native runtime components: add that
inventory before release. It neither builds nor signs nor authorizes publication.
Use a protected signing workflow with narrowly scoped short-lived identity to
attest the exact manifest and artifacts. Verify trusted issuer/subject, repository,
source revision, workflow identity and artifact digests before acceptance; a
checksum or unsigned JSON alone authenticates nothing. The signing mechanism,
OIDC scope, approvals and verifier remain external pending controls.

Retain working logs at most 7 days, restrict access, and privately retain reviewed
redacted evidence as needed. Verify actual settings rather than inferring them
from YAML. Maintainer MFA, minimal package owners, protected main/release refs,
workflow review and a tested private-reporting channel still need owner evidence.
Do not collect passwords, tokens or recovery codes. No proprietary CUDA binaries
may be bundled until redistribution terms have been reviewed and authorized.
