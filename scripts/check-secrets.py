#!/usr/bin/env python3
"""Scan current source and available Git history without printing secret values."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gitleaks", default="gitleaks")
    args = parser.parse_args()
    scanner = str(Path(args.gitleaks).resolve()) if Path(args.gitleaks).is_file() else args.gitleaks
    env = os.environ.copy()
    env.update(GIT_CONFIG_COUNT="1", GIT_CONFIG_KEY_0="safe.directory", GIT_CONFIG_VALUE_0=str(ROOT))
    for key in ("GITLEAKS_CONFIG", "GITLEAKS_CONFIG_TOML"):
        env.pop(key, None)
    git = ["git", "-c", f"safe.directory={ROOT}"]
    files = subprocess.check_output(git + ["ls-files", "-z", "--cached", "--others", "--exclude-standard"], cwd=ROOT)
    commits = subprocess.check_output(git + ["rev-list", "--all", "--count"], cwd=ROOT, text=True).strip()
    with tempfile.TemporaryDirectory(prefix="gudra-secrets-") as folder:
        folder = Path(folder)
        staged = folder / "current"
        staged.mkdir()
        for raw in files.split(b"\0"):
            if not raw:
                continue
            relative = Path(os.fsdecode(raw))
            source = ROOT / relative
            if relative.is_absolute() or ".." in relative.parts or source.is_symlink():
                raise SystemExit("Unsupported source path; secret scan fails closed.")
            if not source.is_file():
                continue
            target = staged / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
        config = folder / "gitleaks.toml"
        config.write_text("[extend]\nuseDefault = true\n", encoding="utf-8")
        ignored = folder / "empty.ignore"
        ignored.write_text("", encoding="utf-8")
        failed = False
        for lane, command in [("current", ["dir", str(staged)]), ("history", ["git", str(ROOT), "--log-opts=--all --full-history"])]:
            report = folder / f"{lane}.json"
            result = subprocess.run([scanner, *command, "--config", str(config), "--gitleaks-ignore-path", str(ignored),
                                     "--ignore-gitleaks-allow", "--redact=100", "--no-banner", "--no-color", "--timeout", "300",
                                     "--report-format", "json", "--report-path", str(report)],
                                    cwd=ROOT, env=env, capture_output=True, text=True, check=False)
            findings = json.loads(report.read_text(encoding="utf-8")) if report.exists() else None
            count = len(findings) if isinstance(findings, list) else "unknown"
            print(f"{lane}: scanner_exit={result.returncode}, findings={count}")
            if result.returncode or count != 0:
                failed = True
                for finding in findings or []:
                    print(json.dumps({key: finding.get(key) for key in ("RuleID", "File", "StartLine", "Commit")}))
        print(f"available_history_commits={commits}")
        return int(failed)


if __name__ == "__main__":
    raise SystemExit(main())
