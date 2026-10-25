#!/usr/bin/env python3
"""Inspect the actual Cargo archive. Does not approve registry GPU publication."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
ALLOWED = {"src", "tests", "examples", "docs"}
ROOT_FILES = {"Cargo.toml", "Cargo.toml.orig", "Cargo.lock", "README.md", "LICENSE",
              "NOTICE", "CHANGELOG.md", "rust-toolchain.toml", ".cargo_vcs_info.json"}
FORBIDDEN = {"target", ".git", ".cutile-cache", "__pycache__", "evidence", "prototypes"}
SUFFIXES = {".cubin", ".ptx", ".tileirbc", ".pyc", ".exe", ".dll", ".so", ".lib",
            ".key", ".pem", ".p12", ".pfx", ".bin"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.archive is None:
        package = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]
        args.archive = ROOT / "target/package" / f"{package['name']}-{package['version']}.crate"
    entries, errors = [], []
    with tarfile.open(args.archive, "r:gz") as archive:
        for member in archive.getmembers():
            path = PurePosixPath(member.name)
            relative = PurePosixPath(*path.parts[1:])
            if path.is_absolute() or ".." in path.parts or not member.isfile():
                errors.append(f"unsafe member: {member.name}")
                continue
            if (set(relative.parts) & FORBIDDEN or relative.suffix.lower() in SUFFIXES
                    or any(part.startswith(".env") for part in relative.parts)
                    or (len(relative.parts) == 1 and relative.name not in ROOT_FILES)
                    or (len(relative.parts) > 1 and relative.parts[0] not in ALLOWED)
                    or (relative.parts[0] == "docs" and relative.suffix != ".md")
                    or member.size > 1024 * 1024):
                errors.append(f"unexpected/generated/oversized member: {member.name}")
            data = archive.extractfile(member).read()
            if b"C:\\Users\\" in data or b"/home/" in data or b"/Users/" in data:
                errors.append(f"private absolute path: {member.name}")
            entries.append({"path": member.name, "bytes": len(data),
                            "sha256": hashlib.sha256(data).hexdigest()})
    if sum(entry["bytes"] for entry in entries) > 5 * 1024 * 1024:
        errors.append("uncompressed archive exceeds 5 MiB")
    report = {"archive": args.archive.name, "sha256": hashlib.sha256(args.archive.read_bytes()).hexdigest(),
              "files": entries, "errors": errors, "pass": not errors,
              "scope": "content hygiene; separate secret scan and dependency safety gates required"}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"{'FAIL' if errors else 'PASS'}: {len(entries)} files, {sum(e['bytes'] for e in entries)} bytes")
    for error in errors:
        print(error)
    return int(bool(errors))


if __name__ == "__main__":
    raise SystemExit(main())
