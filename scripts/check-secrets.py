#!/usr/bin/env python3
"""Scan current source and available Git history without printing secret values."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def scanner_config(root=ROOT):
    """Allow only reviewed public checksum values at their exact manifest paths."""
    policy = json.loads((root / "scripts/secret-scan-policy.json").read_text(encoding="utf-8"))
    lines = ["[extend]", "useDefault = true", ""]
    for entry in policy["public_manifest_checksums"]:
        relative = Path(entry["path"])
        checksum = entry["sha256"]
        if (relative.is_absolute() or ".." in relative.parts
                or not relative.as_posix().startswith("docs/evidence/")
                or relative.name not in {"bundle-manifest.json", "source-manifest.json"}
                or not re.fullmatch(r"[a-f0-9]{64}", checksum)):
            raise SystemExit("Invalid reviewed checksum exception; scan fails closed.")
        path = root / relative
        # Release archives exclude evidence. A missing historical manifest means
        # this exception is not applicable; its exact path still cannot ship.
        if not path.exists():
            continue
        manifest = json.loads(path.read_text(encoding="utf-8"))
        checksums = manifest.get("files", {}) if relative.name == "bundle-manifest.json" else manifest
        if not isinstance(checksums, dict) or checksum not in checksums.values():
            raise SystemExit("Reviewed manifest checksum changed; re-triage before scanning.")
        path_regex = r"(^|[/\\])" + re.escape(relative.as_posix()).replace("/", r"[/\\]") + "$"
        lines += ["[[allowlists]]", 'description = "Reviewed public source-manifest SHA-256; exact path AND exact value"',
                  'targetRules = ["generic-api-key"]', 'condition = "AND"', 'regexTarget = "secret"',
                  "paths = [" + json.dumps(path_regex) + "]",
                  "regexes = [" + json.dumps("^" + checksum + "$") + "]", ""]
    return "\n".join(lines)


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
    scratch = ROOT / "target" / "secret-scan"
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="gudra-secrets-", dir=scratch) as folder:
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
        config.write_text(scanner_config(), encoding="utf-8")
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
