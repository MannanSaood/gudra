#!/usr/bin/env python3
"""Create a deterministic curated source snapshot; publication is never authorized."""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path
import re
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]
TOP = {"src", "tests", "examples", "vendor", "scripts", ".github", "templates", "fixtures"}
FILES = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "deny.toml", "README.md",
         "LICENSE", "NOTICE", "CHANGELOG.md", "SECURITY.md", "CONTRIBUTING.md",
         "CODE_OF_CONDUCT.md", ".gitignore", ".gitattributes"}
BAD = {"target", ".git", "__pycache__", ".cutile-cache", "evidence", "prototypes"}
BINARY = {".exe", ".dll", ".so", ".a", ".lib", ".bin", ".cubin", ".ptx", ".tileirbc",
          ".pyc", ".pem", ".key", ".p12", ".pfx"}


def selected(relative):
    return (relative.as_posix() in FILES or relative.parts[0] in TOP
            or (relative.parts[0] == "docs" and relative.suffix in {".md", ".json"}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--stage", type=Path)
    parser.add_argument("--require-clean", action="store_true")
    args = parser.parse_args()
    git = ["git", "-c", f"safe.directory={ROOT}"]
    revision = subprocess.check_output(git + ["rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    dirty = bool(subprocess.check_output(git + ["status", "--porcelain"], cwd=ROOT))
    if args.require_clean and dirty:
        raise SystemExit("FAIL: require clean committed source")
    paths = subprocess.check_output(git + ["ls-files", "-z", "--cached", "--others", "--exclude-standard"], cwd=ROOT)
    args.output.mkdir(parents=True, exist_ok=True)
    if args.stage:
        stage = args.stage.resolve()
        if not stage.is_relative_to((ROOT / "target").resolve()) or stage.exists():
            raise SystemExit("FAIL: stage must be new and under checkout/target")
        stage.mkdir(parents=True)
    entries, payloads = [], []
    for name in sorted(set(paths.decode("utf-8").split("\0"))):
        if not name:
            continue
        relative = Path(name)
        if not selected(relative) or set(relative.parts) & BAD:
            continue
        path = ROOT / relative
        if relative.is_absolute() or ".." in relative.parts or path.is_symlink() or not path.is_file():
            raise SystemExit(f"FAIL: invalid source member {name}")
        if relative.suffix.lower() in BINARY or any(p.startswith(".env") for p in relative.parts):
            raise SystemExit(f"FAIL: prohibited release member {name}")
        data = path.read_bytes()
        if len(data) > 1024 * 1024:
            raise SystemExit(f"FAIL: oversized source member {name}")
        if re.search(rb"(?:[A-Za-z]:[\\/]Users[\\/]|/(?:home|Users)/)[A-Za-z0-9_.-]+", data):
            raise SystemExit(f"FAIL: private absolute path in {name}")
        executable = relative.suffix in {".py", ".sh"}
        entries.append({"path": name, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(),
                        "mode": "0755" if executable else "0644"})
        payloads.append((name, data, executable))
        if args.stage:
            target = stage / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
    if sum(e["bytes"] for e in entries) > 10 * 1024 * 1024:
        raise SystemExit("FAIL: source exceeds 10 MiB")
    tree_digest = hashlib.sha256(json.dumps(entries, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    prefix = f"gudra-source-{tree_digest[:16]}"
    archive = args.output / f"{prefix}.tar.gz"
    with archive.open("wb") as destination:
        with gzip.GzipFile(filename="", mode="wb", fileobj=destination, mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w") as tar:
                for name, data, executable in payloads:
                    member = tarfile.TarInfo(f"{prefix}/{name}")
                    member.size, member.mode, member.mtime = len(data), 0o755 if executable else 0o644, 0
                    tar.addfile(member, io.BytesIO(data))
    result = {"base_revision": revision, "dirty": dirty, "tree_sha256": tree_digest,
              "archive": archive.name, "archive_sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
              "signed": False, "publication_authorized": False, "gpu_verified": False,
              "files": entries, "intentional_exclusions": ["historical evidence/benchmark dumps", "halo prototypes", "caches", "generated/native artifacts", "credentials", "local tools"]}
    (args.output / "source-inventory.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    (args.output / "SHA256SUMS").write_text(f"{result['archive_sha256']}  {archive.name}\n", encoding="utf-8")
    print(f"PASS: {len(entries)} curated files; dirty={dirty}; archive={archive.name}")


if __name__ == "__main__":
    main()
