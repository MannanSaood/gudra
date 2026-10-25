#!/usr/bin/env python3
"""Verify CPU onboarding from a fresh source copy and empty build directory.

Uses installed toolchains and cached dependencies; does not install software.
Recorded logs replace local paths. Inspect them before public sharing.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cargo", default="cargo")
    parser.add_argument("--rustc", default="rustc")
    parser.add_argument("--rustdoc", default="rustdoc")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env.update(CARGO=args.cargo, RUSTC=args.rustc, RUSTDOC=args.rustdoc)
    records = []
    scratch = ROOT / "target" / "doc-walkthrough"
    scratch.mkdir(parents=True, exist_ok=True)
    if not scratch.resolve().is_relative_to(ROOT.resolve()):
        raise SystemExit("Walkthrough scratch must stay inside the checkout.")
    with tempfile.TemporaryDirectory(prefix="clean-", dir=scratch) as folder:
        source = Path(folder) / "source"
        source.mkdir()
        for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml"):
            shutil.copyfile(ROOT / name, source / name)
        for name in ("src", "tests", "examples", "vendor"):
            shutil.copytree(ROOT / name, source / name,
                            ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
        (source / "scripts").mkdir()
        shutil.copyfile(ROOT / "scripts/check-ui.py", source / "scripts/check-ui.py")
        manifest = {p.relative_to(source).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                    for p in sorted(source.rglob("*")) if p.is_file()}
        (output / "source-manifest.json").write_text(
            json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        env["CARGO_TARGET_DIR"] = str(source / "target")
        # Preserve diagnostics while removing paths that identify this machine.
        replacements = [(str(Path.home()), "<HOME>"), (str(ROOT), "<CHECKOUT>"),
                        (str(source), "<CLEAN_SOURCE>"), (str(Path(folder)), "<TEMP>")]
        replacements.sort(key=lambda item: len(item[0]), reverse=True)
        commands = [
            ("cargo-version", [args.cargo, "--version"]),
            ("rustc-version", [args.rustc, "--version"]),
            ("build", [args.cargo, "build", "--locked", "--offline"]),
            ("tests", [args.cargo, "test", "--locked", "--offline"]),
            ("release-tests", [args.cargo, "test", "--locked", "--offline", "--release"]),
            ("cpu-step", [args.cargo, "run", "--locked", "--offline", "--example", "cpu_step"]),
            ("curvature", [args.cargo, "run", "--locked", "--offline", "--release",
                           "--example", "laplacian_curvature"]),
            ("residual", [args.cargo, "run", "--locked", "--offline", "--release",
                          "--example", "poisson_residual"]),
            ("ownership", [sys.executable, "scripts/check-ui.py"]),
            ("api-docs", [args.cargo, "doc", "--locked", "--offline", "--no-deps"]),
        ]
        for name, command in commands:
            run = subprocess.run(command, cwd=source, env=env, capture_output=True,
                                 text=True, errors="replace", check=False)
            log = run.stdout + run.stderr
            display = " ".join(command)
            for original, replacement in replacements:
                log = log.replace(original, replacement).replace(original.replace("\\", "/"), replacement)
                display = display.replace(original, replacement)
            (output / f"{name}.txt").write_text(log, encoding="utf-8")
            records.append({"check": name, "command": display, "exit_code": run.returncode})
            print(f"{name}: exit={run.returncode}", flush=True)
        summary = {"fresh_source": True, "empty_target_at_start": True,
                   "installed_toolchain_and_cached_dependencies": True,
                   "independent_reviewer": False, "gpu_executed": False,
                   "source_manifest_sha256": hashlib.sha256(
                       (output / "source-manifest.json").read_bytes()).hexdigest(),
                   "checks": records}
        (output / "results.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    return int(any(item["exit_code"] for item in records))


if __name__ == "__main__":
    raise SystemExit(main())
