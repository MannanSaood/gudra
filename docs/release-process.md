# Release evidence and provenance

Gudra remains non-publishable (`publish = false`). The release-evidence workflow
does not create a GitHub release or publish a package. It produces reviewable
evidence for an explicitly approved immutable commit on protected `main`.

Before dispatching the workflow, require the CPU/security checks and the separate
isolated GPU gate for the same commit. Dispatch **Release evidence** from `main`
and enter the complete 40-character approved commit SHA. The workflow rejects a
different ref, abbreviated SHA, dirty checkout, or revision mismatch.

The clean hosted worker reruns formatting, Clippy, debug/release tests, docs,
advisory/license/source policy, vendor verification, installer tests, and the
private-temp launcher tests. It then generates:

- a deterministic gzip-compressed source archive from the approved Git tree;
- SHA-256 checksums;
- a CycloneDX 1.5 SBOM for the complete locked Rust dependency graph, including
  vendored path dependencies and their declared licenses;
- Rust, Cargo, Python, Git, OS and executable identities; and
- a manifest binding those records to the source revision and vendor manifest.

CUDA, the NVIDIA driver, and `tileiras` are external prerequisites. They are not
included in the repository or source archive and are not redistributed by this
process. The separately reviewed GPU evidence records their versions.

GitHub signs Sigstore-backed SLSA provenance for every generated file using a
short-lived OIDC identity. The workflow receives no release secret and grants
only `contents: read`, `id-token: write`, `attestations: write`, and
`artifact-metadata: write`. Evidence artifacts expire after 14 days; attestations
remain bound to their subject digests in GitHub's attestation service.

After downloading the evidence, verify checksums and provenance before any
separately authorized publication:

```sh
cd release-evidence-<revision>
sha256sum --check SHA256SUMS
gh attestation verify "gudra-<revision>.tar.gz" -R MannanSaood/gudra
gh attestation verify sbom.cdx.json -R MannanSaood/gudra
gh attestation verify manifest.json -R MannanSaood/gudra
```

Publication requires a separate owner decision after repository protections,
private reporting, maintainer account protection, package ownership, and the
recorded release gate have all been verified. Never upload PR-produced binaries,
PTX, cubins, or benchmark artifacts to a release or privileged GPU worker.
