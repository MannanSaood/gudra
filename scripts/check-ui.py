#!/usr/bin/env python3
"""Type-check real public APIs; no CUDA execution, mocks, or stderr snapshots."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def messages(text):
    return [json.loads(line) for line in text.splitlines() if line.startswith("{")]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gpu", action="store_true", help="also check actual GPU APIs (toolkit required)")
    args = parser.parse_args()
    cargo = os.environ.get("CARGO", "cargo")
    rustc = os.environ.get("RUSTC", "rustc")
    command = [cargo, "check", "--locked", "--offline", "--lib", "--message-format=json"]
    if args.gpu:
        command += ["--features", "gpu"]
    build = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)
    if build.returncode:
        print(build.stderr, file=sys.stderr)
        for message in messages(build.stdout):
            if message.get("reason") == "compiler-message":
                print(message["message"].get("rendered", ""), file=sys.stderr)
        print("BLOCKED: library did not compile; no UI fixture counted as passed.", file=sys.stderr)
        return 1
    artifacts = [message for message in messages(build.stdout)
                 if message.get("reason") == "compiler-artifact"
                 and message["target"]["name"] == "gudra"]
    metadata = [Path(name) for artifact in artifacts for name in artifact["filenames"]
                if name.endswith(".rmeta")]
    if len(metadata) != 1:
        raise RuntimeError(f"expected current build's single gudra metadata file: {metadata}")
    # Use Cargo's exact artifact, never a stale glob from a different feature set.
    deps = metadata[0].parent
    output = deps.parent / "ui"
    output.mkdir(exist_ok=True)
    lanes = ["cpu", "gpu"] if args.gpu else ["cpu"]
    failures = 0
    for lane in lanes:
        fixtures = sorted((ROOT / "tests" / "ui" / lane).glob("*.rs"))
        assert fixtures, f"no fixtures for {lane}"
        # Positive control must compile before any rejection can be evidence.
        fixtures.sort(key=lambda path: (path.stem != "pass", path.name))
        for fixture in fixtures:
            expected = re.findall(r"// error: (E\d{4})", fixture.read_text())
            result = subprocess.run([
                rustc, str(fixture), "--crate-name", f"ui_{lane}_{fixture.stem}",
                "--crate-type=lib", "--edition=2021", "--emit=metadata",
                "--error-format=json", "--extern", f"gudra={metadata[0]}",
                "-L", f"dependency={deps}", "--out-dir", str(output),
            ], cwd=ROOT, capture_output=True, text=True, check=False)
            errors = [item for item in messages(result.stderr) if item.get("level") == "error"]
            coded = [item for item in errors if item.get("code")]
            actual = [item["code"]["code"] for item in coded]
            located = all(any(span["is_primary"] and Path(span["file_name"]).resolve() == fixture.resolve()
                              for span in item["spans"]) for item in coded)
            # rustc's uncoded final 'aborting due to ...' summary is allowed;
            # linker/import/backend errors with no fixture span are not evidence.
            uncoded = [item for item in errors if not item.get("code")
                       and not item["message"].startswith("aborting due to")]
            passed = (sorted(actual) == sorted(expected) and located and not uncoded
                      and (result.returncode != 0) == bool(expected))
            print(f"{'PASS' if passed else 'FAIL'} {lane}/{fixture.name}: {actual or 'compiles'}")
            (output / f"{lane}_{fixture.stem}.jsonl").write_text(result.stderr, encoding="utf-8")
            if not passed:
                failures += 1
                print(result.stderr, file=sys.stderr)
                if not expected:
                    break
    return bool(failures)


if __name__ == "__main__":
    sys.exit(main())
