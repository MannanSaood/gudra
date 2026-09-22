#!/usr/bin/env python3
"""Reject drift from reviewed vendored compiler content."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
manifest = json.loads((root/'vendor/checksums.json').read_text())
if set(manifest) != {'cutile-compiler'}:
    raise SystemExit('FAIL: unexpected vendor package')
for name, files in manifest.items():
    base = root/'vendor'/name
    actual = {}
    for path in base.rglob('*'):
        if path.is_symlink():
            raise SystemExit('FAIL: vendor symlink')
        if path.is_file():
            actual[path.relative_to(base).as_posix()] = hashlib.sha256(path.read_bytes().replace(b'\r\n', b'\n')).hexdigest()
    if actual != {path: value['sha256'] for path, value in files.items()}:
        raise SystemExit('FAIL: vendor digest/file set changed; review required')
print('PASS: vendored source matches reviewed manifest')
