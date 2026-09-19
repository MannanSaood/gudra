#!/usr/bin/env python3
"""Fail on unexpected dependency sources or executable build capabilities."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"


def inventory(metadata):
    checksums = {(p["name"], p["version"]): p.get("checksum")
                 for p in tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))["package"]}
    result = []
    for package in metadata["packages"]:
        if package["id"] in metadata["workspace_members"] and package["name"] == "gudra":
            continue
        if package.get("source") != REGISTRY or not checksums.get((package["name"], package["version"])):
            raise SystemExit("Unexpected Git/path/registry dependency or missing checksum.")
        base = Path(package["manifest_path"]).parent
        kinds = {kind for target in package["targets"] for kind in target["kind"]}
        result.append({
            "name": package["name"], "version": package["version"], "source": package["source"],
            "checksum": checksums[(package["name"], package["version"])], "license": package.get("license"),
            "build_script": "custom-build" in kinds, "proc_macro": "proc-macro" in kinds,
            "links": package.get("links"),
            "native_files": sorted(str(path.relative_to(base)).replace("\\", "/") for path in base.rglob("*")
                                   if path.suffix in (".c", ".cc", ".cpp", ".cu", ".S", ".s", ".asm", ".o", ".a", ".lib", ".dll", ".so", ".dylib") and path.is_file()),
        })
    return sorted(result, key=lambda p: (p["name"], p["version"]))


def main():
    cargo = os.environ.get("CARGO", "cargo")
    metadata = json.loads(subprocess.check_output([cargo, "metadata", "--locked", "--all-features", "--format-version", "1"], cwd=ROOT))
    actual = inventory(metadata)
    expected = json.loads((ROOT / "docs/security/dependency-inventory.json").read_text(encoding="utf-8"))
    if actual != expected:
        raise SystemExit("Dependency inventory changed: review licenses, native/build/proc-macro authority and source digests, then update the reviewed inventory.")
    print(f"PASS: {len(actual)} checksummed registry dependencies; no Git/path dependencies; capability inventory unchanged.")


if __name__ == "__main__":
    main()
