#!/usr/bin/env python3
"""Fail closed unless the recorded gate passes for the current source/configuration."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def source_digest():
    paths = [ROOT / name for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "deny.toml")]
    for directory in ("src", "tests", "scripts", ".github"):
        paths.extend(p for p in (ROOT / directory).rglob("*") if p.is_file()
                     and "__pycache__" not in p.parts and p.suffix != ".pyc")
    digest = hashlib.sha256()
    for path in sorted(paths, key=lambda p: p.relative_to(ROOT).as_posix()):
        digest.update(path.relative_to(ROOT).as_posix().encode() + b"\0")
        digest.update(path.read_bytes().replace(b"\r\n", b"\n") + b"\0")
    return digest.hexdigest()


def main():
    gate = json.loads((ROOT / "docs/security/release-gate.json").read_text(encoding="utf-8"))
    if gate.get("source_sha256") != source_digest():
        raise SystemExit("FAIL: source/configuration changed since the recorded release review.")
    required = gate.get("required_checks", [])
    pending = [item["id"] for item in required if item.get("status") != "pass"]
    if gate.get("decision") != "pass" or not required or pending:
        print("FAIL: release is not approved. Open checks: " + ", ".join(pending))
        return 1
    print("PASS: recorded required checks pass for this source digest; verify external release authorization.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
