# Release provenance and verification

Publication remains disabled. Release evidence was generated from approved
immutable commit `c5f3322515834ea646e191acede544241e9a4328` in clean GitHub
Actions run 36244888865 without release credentials.
Never promote PR-produced binaries, cubins, PTX or benchmark artifacts. Verify
source review, required checks and protected tag/environment approval first.

Run cargo build --locked --release with the approved features. Record OS image,
rustc -Vv, cargo -V, CUDA toolkit, tileiras, loaded native libraries, driver/GPU
and every native tool digest. Supply only the intended artifacts to:

```sh
python3 scripts/release-manifest.py --artifact target/release/APP --tool /path/to/rustc --tool /path/to/tileiras --output target/release-evidence
sha256sum -c target/release-evidence/SHA256SUMS
```

The helper emits a CycloneDX inventory for the complete locked Rust graph,
checksums and a manifest binding the source revision, lockfile, vendored source,
tools and artifact digests. CUDA, the NVIDIA driver and tileiras are recorded as
external prerequisites and are not redistributed. The protected workflow uses a
narrowly scoped short-lived OIDC identity to attest the archive, SBOM and manifest.
The downloaded files passed SHA-256 verification and all three attestations
verified with GitHub CLI 2.101.0. Evidence is retained for 14 days.

Repository settings verified 14-day log/artifact retention, maintainer MFA,
minimal ownership, protected main and release refs, restricted workflows and a
working private-reporting channel. No passwords, tokens or recovery codes were
collected. The release process does not bundle proprietary CUDA components.
Publication requires a separate explicit owner instruction after rechecking that
the bound revision and these controls remain current.
