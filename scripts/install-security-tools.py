#!/usr/bin/env python3
"""Install reviewed audit binaries locally without extracting archive paths."""
import hashlib
import io
from pathlib import Path
import platform
import sys
import tarfile
import urllib.request

TOOLS = [
    ("cargo-deny", "https://github.com/EmbarkStudios/cargo-deny/releases/download/0.20.2/cargo-deny-0.20.2-x86_64-unknown-linux-musl.tar.gz", "9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f"),
    ("gitleaks", "https://github.com/gitleaks/gitleaks/releases/download/v8.30.1/gitleaks_8.30.1_linux_x64.tar.gz", "551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb"),
]


def verified_binary(payload, name, expected):
    if hashlib.sha256(payload).hexdigest() != expected:
        raise ValueError(f"Checksum mismatch for {name}; nothing executed.")
    with tarfile.open(fileobj=io.BytesIO(payload)) as archive:
        members = [m for m in archive.getmembers() if Path(m.name).name == name and m.isfile()]
        if len(members) != 1:
            raise ValueError(f"Ambiguous binary in {name} archive.")
        return archive.extractfile(members[0]).read()


def main():
    if sys.platform != "linux" or platform.machine() not in ("x86_64", "amd64"):
        raise SystemExit("Pinned installer supports Linux x86_64 only.")
    destination = Path(sys.argv[1] if len(sys.argv) > 1 else "target/security-tools")
    destination.mkdir(parents=True, exist_ok=True)
    for name, url, expected in TOOLS:
        with urllib.request.urlopen(url, timeout=60) as response:
            payload = response.read(64 * 1024 * 1024 + 1)
        binary = verified_binary(payload, name, expected)
        path = destination / name
        path.write_bytes(binary)
        path.chmod(0o755)
        print(f"Verified {name}: sha256 {expected}")


if __name__ == "__main__":
    main()
