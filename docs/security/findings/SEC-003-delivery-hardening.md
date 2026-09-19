# Security Finding

## Metadata

- ID: SEC-003
- Title: Make delivery authority and dependency policy explicit
- Status: mitigated in repository; external settings verification pending
- Severity: informational
- Affected versions/components: baseline .github/workflows/rust.yml and absent security policy
- Reporter/date: independent local audit, 2026-09-19

## Summary

The original hosted CPU workflow used checkout@v4, inherited token permissions and credential persistence, and omitted --locked. No release secret, privileged GPU runner or executable artifact import was present. This was a hardening gap, not evidence of credential theft or privilege escalation.

## Threat model

- Attacker capability: proposed PR code or compromised mutable action.
- Required access: workflow execution; additional token rights depend on unverified settings.
- Trust boundary crossed: contribution/dependency code to runner and potential delivery authority.
- Assets affected: CI credentials, source/artifact integrity.

## Evidence and reproduction

Read baseline workflow at 22f7ff1 and current replacement. No hostile workflow was executed.

## Impact

Supply-chain compromise can be severe when credentials or sensitive runners are present. Those prerequisites were not established here; informational severity avoids inventing them.

## Root cause

No explicit least-privilege/pinning policy or automated security lane.

## Remediation

- Immediate containment/code fix: immutable checkout SHA, contents:read, persist-credentials:false, hosted Ubuntu, timeouts, locked build commands, no artifact imports/uploads or release credentials.
- Regression/check: cargo-deny all-feature advisory/license/source checks, reviewed dependency capability snapshot, Gitleaks current/history with redaction, weekly checks and update proposals.
- Policy: SECURITY.md, private reporting/release ownership/provenance follow-ups, 7-day restricted log policy requiring external settings verification.

## Verification

RustSec/license/bans/sources checks pass; Gitleaks current/history passes. All 41 build-script target sources and 10 proc-macro entries received capability review. Workflow reviewed locally; remote CI and external repository settings not exercised.

## Disclosure

No external changes/publication made. No CVE claim.
