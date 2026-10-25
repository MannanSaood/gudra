# Provenance, CI and release operations

## Evidence and signing

For an explicitly approved immutable commit on protected main, dispatch the
existing Release evidence workflow with its full SHA. The source digest guard
must pass before any OIDC attestation. CPU/security and isolated GPU evidence
must refer to that same source; the worker must not import executable artifacts
from a PR. The workflow builds a curated source archive retaining vendor fixes,
checks its inventory, emits a complete locked Rust CycloneDX SBOM, tool executable
identities and manifest, and calculates SHA-256 for every artifact. CUDA/driver/
tileiras versions and executable identities belong in the separate GPU record.

GitHub Sigstore/SLSA attestations use short-lived OIDC, not a stored signing key.
Verify `sha256sum --check SHA256SUMS` and `gh attestation verify <artifact> -R
MannanSaood/gudra` for source, SBOM and manifest, then match the attested SHA and
workflow/ref to the approved commit. Checksums authenticate bytes only when the
checksum file/provenance origin is verified. Local Chat 12 archives are unsigned
dirty snapshots; never describe them as signed/reproducible commit releases.
Keep verified artifacts and gate/GPU evidence beyond the 14-day Actions retention
in maintainer-controlled release storage after authorized publication.

Root workflow permissions remain read-only/empty; checkout does not persist
credentials. Only the environment-protected evidence job grants attestation/OIDC
permissions. Actions are pinned to reviewed full SHAs; no third-party action,
release token, publishing secret or self-hosted runner is used. Untrusted PRs
run only on disposable hosted CPU workers under `pull_request`, never
`pull_request_target`. A PR may change scripts but cannot receive secrets,
OIDC or a privileged GPU runner. Environment protection and main SHA approval
are administrative prerequisites. Audit pin updates and verify publisher SHA
before merging them. Any future publisher must have a separate protected
environment and exact release approval, with least-scope short-lived credentials
or trusted publishing when configured; do not store credentials in files/artifacts.

## Rollback and yanking

Keep the last verified source SHA, lockfile, vendor manifest, artifact digests and
attestations. On a bad GitHub release, stop promotion, mark it withdrawn with a
reason and replacement link, and preserve the original immutable tag and evidence.
Do not force-move a tag or silently replace bytes. Consumers revert to the last
verified source/lockfile and rebuild locally; discard generated JIT/cache artifacts
when their compiler/backend identities changed. Do not load untrusted cubins.

If a future crates.io version is defective, the authorized crate owner runs
`cargo yank --vers <version> gudra` and records the incident/version; yanking
does not delete source or remove existing lockfile selections. Ship a fixed new
version and guidance for affected lockfiles. `cargo yank --undo` is a separate
reviewed owner action after a mistaken yank. No yank applies to this unpublished
`0.0.0` development candidate. Do not claim automatic runtime rollback; GPU faults
require replacing the aborted worker and reinitializing trusted state.

## Security response

Use SECURITY.md and private GitHub advisory reporting. The repository owner is
the accountable responder under solo-maintainer governance. Privately record
affected source SHAs/versions/configurations and reproduce on an isolated worker;
preserve redacted logs/digests, disable affected release promotion, assess severity
and scope, and prepare a reviewed fix plus regression/GPU tests. Rotate an actually
exposed credential through its owner, never paste it into reports. Coordinate
upstream privately when appropriate; public advisories/disclosure, notifications
and releases require owner authorization. Record remediation and affected-version
guidance, then rerun every invalidated gate and publish a new immutable version.
