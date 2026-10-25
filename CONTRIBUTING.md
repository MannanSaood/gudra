# Contributing

Start with [the README](README.md), [setup](docs/setup.md) and
[architecture/roadmap](docs/architecture.md). Small reproducible failures and
scientific workload descriptions are more useful than broad feature lists.
The project is experimental, MIT licensed and currently maintained under the
solo-maintainer policy in [SECURITY.md](SECURITY.md).

Use the bug, performance, safety-boundary or environment issue template. A
suspected vulnerability must go to [private reporting](https://github.com/MannanSaood/gudra/security/advisories/new),
not a public issue. Review [what to include and redact](docs/community/reporting.md).
For a feature, describe layout, boundary policy, scale, precision and an actual
application; relate it to an existing roadmap item before implementing an abstraction.

Create a focused branch/PR from the documented revision, preserve Cargo.lock and
vendor notices, explain the behavior change and cite validation. Run:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --release
python3 scripts/check-ui.py
cargo doc --locked --no-deps
```

GPU changes additionally need the [candidate evidence runner](docs/gpu-candidate-testing.md)
on a trusted Linux CUDA worker. Mark unavailable checks BLOCKED/NOT RUN;
CPU results do not stand in for GPU results. Follow the isolated-worker policy
before device-fault tests. Changes to unsafe dependencies or completion require
review of the [safety boundary](docs/safety-model.md) and security evidence.

Documentation PRs should check commands/links, runnable examples and privacy.
Do not bundle raw workstation logs, credentials, generated CUDA binaries or
private data. Describe benchmark scope and regressions honestly. No publication
or release action follows automatically from an accepted PR.

The project uses the [code of conduct](CODE_OF_CONDUCT.md). The local decision
for this package is a short project-specific conduct policy with existing GitHub
moderation paths; no independent conduct team or unconfigured private inbox is
claimed. The maintainer must confirm ownership of enforcement before outreach.
