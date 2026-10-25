#!/usr/bin/env python3
"""Exercise the real scanner: exact public hashes only, never paths alone."""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gitleaks", required=True)
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location("secret_checks", ROOT / "scripts/check-secrets.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    entry = json.loads((ROOT / "scripts/secret-scan-policy.json").read_text())["public_manifest_checksums"][0]
    scratch = ROOT / "target" / "secret-policy-tests"
    scratch.mkdir(parents=True, exist_ok=True)
    scanner = str(Path(args.gitleaks).resolve())
    with tempfile.TemporaryDirectory(dir=scratch) as directory:
        folder = Path(directory)
        config = folder / "config.toml"
        # Build a synthetic policy root to prove validation rejects changed
        # manifest values without mutating historical evidence.
        policy_root = folder / "policy-root"
        (policy_root / "scripts").mkdir(parents=True)
        (policy_root / "scripts/secret-scan-policy.json").write_text(json.dumps({"public_manifest_checksums": [entry]}))
        manifest = policy_root / entry["path"]
        manifest.parent.mkdir(parents=True)
        manifest.write_text(json.dumps({"files": {"src/key.rs": entry["sha256"]}}))
        config.write_text(module.scanner_config(policy_root))
        manifest.write_text(json.dumps({"files": {"src/key.rs": "f" * 64}}))
        try:
            module.scanner_config(policy_root)
        except SystemExit:
            pass
        else:
            raise AssertionError("changed reviewed checksum must fail closed")
        source = folder / "scan"
        path = source / entry["path"]
        path.parent.mkdir(parents=True)
        key = "api" + "_key"
        # Synthetic values are generated from public digests; no credential exists.
        other = __import__("hashlib").sha256(b"synthetic scanner negative control").hexdigest()
        path.write_text(f'{key} = "{entry["sha256"]}"\n{key} = "{other}"\n')
        (source / "unrelated.txt").write_text(f'{key} = "{entry["sha256"]}"\n')
        ignored = folder / "empty.ignore"
        ignored.write_text("")
        report = folder / "report.json"
        result = subprocess.run([scanner, "dir", str(source), "--config", str(config),
            "--gitleaks-ignore-path", str(ignored), "--ignore-gitleaks-allow", "--redact=100",
            "--no-banner", "--no-color", "--report-format", "json", "--report-path", str(report)],
            capture_output=True, text=True, check=False)
        findings = json.loads(report.read_text())
        assert result.returncode == 1 and len(findings) == 2, (
            f"expected two negative-control findings; exit={result.returncode}; "
            f"findings={[(Path(f['File']).name, f['StartLine'], f['RuleID']) for f in findings]}")
        assert {f["StartLine"] for f in findings if f["File"].endswith("bundle-manifest.json")} == {2}
    print("PASS: exact public checksum suppressed; changed value/unrelated path detected; stale checksum rejected")


if __name__ == "__main__":
    main()
