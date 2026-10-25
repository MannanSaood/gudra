#!/usr/bin/env python3
"""Capture local release dry runs, with explicit toolchain limitations and no publish."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cargo", default="cargo")
    parser.add_argument("--rustc", default="rustc")
    parser.add_argument("--rustdoc", default="rustdoc")
    parser.add_argument("--lint-cargo", default="cargo")
    parser.add_argument("--lint-rustc", default="rustc")
    parser.add_argument("--deny", default="cargo-deny")
    parser.add_argument("--gitleaks", default="gitleaks")
    parser.add_argument("--toolchain", default="stable", help="rustup selection for Cargo subcommands")
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env.update(CARGO=args.cargo, RUSTC=args.rustc, RUSTDOC=args.rustdoc,
               RUSTUP_TOOLCHAIN=args.toolchain,
               GIT_CONFIG_COUNT="1", GIT_CONFIG_KEY_0="safe.directory", GIT_CONFIG_VALUE_0=str(ROOT))
    lint = env.copy()
    lint.update(CARGO=args.lint_cargo, RUSTC=args.lint_rustc, RUSTUP_TOOLCHAIN=args.toolchain)
    commands = [
        ("cargo-version", [args.cargo, "-V"], env),
        ("rustc-version", [args.rustc, "-Vv"], env),
        ("lint-version", [args.lint_rustc, "-Vv"], lint),
        ("format", [args.lint_cargo, "fmt", "--all", "--", "--check"], lint),
        ("clippy", [args.lint_cargo, "clippy", "--locked", "--offline", "--all-targets", "--", "-D", "warnings"], lint),
        ("build", [args.cargo, "build", "--locked", "--offline"], env),
        ("tests", [args.cargo, "test", "--locked", "--offline"], env),
        ("release-tests", [args.cargo, "test", "--locked", "--offline", "--release"], env),
        ("doctests", [args.cargo, "test", "--locked", "--offline", "--doc"], env),
        ("ui", [sys.executable, "scripts/check-ui.py"], env),
        ("docs", [args.cargo, "doc", "--locked", "--offline", "--no-deps"], dict(env, RUSTDOCFLAGS="-D warnings")),
        ("vendor", [sys.executable, "scripts/check-vendor.py"], env),
        ("dependency-policy", [sys.executable, "scripts/check-dependency-policy.py"], env),
        ("security-tools", [sys.executable, "scripts/test-security-tools.py"], env),
        ("advisory-license-policy", [args.deny, "--all-features", "--locked", "check"], env),
        ("secrets", [sys.executable, "scripts/check-secrets.py", "--gitleaks", args.gitleaks], env),
        ("package-list", [args.cargo, "package", "--locked", "--offline", "--allow-dirty", "--list"], env),
        ("package-dry-run", [args.cargo, "package", "--locked", "--offline", "--allow-dirty"], env),
        ("package-contents", [sys.executable, "scripts/check-package.py", "--output", str(out / "package-contents.json")], env),
        ("gpu-build", [args.cargo, "check", "--locked", "--offline", "--features", "gpu", "--all-targets"], env),
        ("gpu-tests", [args.cargo, "test", "--locked", "--offline", "--features", "gpu", "--test", "gpu_jacobi", "--test", "gpu_laplacian", "--", "--test-threads=1"], env),
        ("gpu-ui", [sys.executable, "scripts/check-ui.py", "--gpu"], env),
        ("final-security-gate", [sys.executable, "scripts/check-release-gate.py"], env),
    ]
    records = []
    replacements = sorted([(str(ROOT), "<CHECKOUT>"), (str(Path.home()), "<HOME>")], key=lambda p: len(p[0]), reverse=True)
    for name, command, check_env in commands:
        try:
            result = subprocess.run(command, cwd=ROOT, env=check_env, capture_output=True,
                                    text=True, errors="replace", timeout=300, check=False)
            log, code = result.stdout + result.stderr, result.returncode
        except (OSError, subprocess.TimeoutExpired) as error:
            log, code = str(error), -1
        display = " ".join(command)
        for original, replacement in replacements:
            for spelling in (original, original.replace("\\", "/")):
                log = log.replace(spelling, replacement)
                display = display.replace(spelling, replacement)
        (out / f"{name}.txt").write_text(log, encoding="utf-8")
        records.append({"check": name, "command": display, "exit_code": code})
        print(f"{name}: exit={code}", flush=True)
    (out / "results.json").write_text(json.dumps({"checks": records, "publication_authorized": False,
        "dirty_development_snapshot": True, "local_gpu_execution_verified": False}, indent=2) + "\n", encoding="utf-8")
    return int(any(r["exit_code"] != 0 for r in records))


if __name__ == "__main__":
    raise SystemExit(main())
