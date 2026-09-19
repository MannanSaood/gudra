#!/usr/bin/env python3
"""Run sanitizer checks on freshly built GPU tests, without distro/version assumptions."""
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, check=True)
    subprocess.run(["compute-sanitizer", "--version"], cwd=ROOT, check=True)
    build = subprocess.run(
        ["cargo", "build", "--locked", "--features", "gpu", "--tests", "--message-format=json"],
        cwd=ROOT, stdout=subprocess.PIPE, text=True, check=False,
    )
    artifacts = []
    for line in build.stdout.splitlines():
        if not line.startswith("{"):
            continue
        message = json.loads(line)
        if message.get("reason") == "compiler-message":
            print(message["message"].get("rendered", ""), flush=True)
        elif message.get("reason") == "compiler-artifact":
            artifacts.append(message)
    build.check_returncode()
    executables = {
        artifact["target"]["name"]: artifact["executable"]
        for artifact in artifacts
        if artifact.get("executable") and artifact["profile"]["test"]
        and artifact["target"]["name"] in ("gudra", "gpu_jacobi")
    }
    if set(executables) != {"gudra", "gpu_jacobi"}:
        raise RuntimeError(f"Missing GPU test executables: {executables}")
    for name, executable in sorted(executables.items()):
        for tool in ("memcheck", "initcheck"):
            command = ["compute-sanitizer", "--tool", tool, "--error-exitcode", "99"]
            if tool == "memcheck":
                command += ["--track-stream-ordered-races", "all"]
            command += [executable, "--test-threads=1"]
            print(f"RUN {name}: {tool}", flush=True)
            subprocess.run(command, cwd=ROOT, check=True)
    print("PASS: both GPU test executables passed memcheck and initcheck.", flush=True)


if __name__ == "__main__":
    main()
