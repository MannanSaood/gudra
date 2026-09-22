#!/usr/bin/env python3
"""Linux launcher regression tests; fails visibly on unsupported hosts."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

if sys.platform != 'linux':
    raise SystemExit('FAIL: launcher tests require Linux')
launcher = Path(__file__).with_name('run-private-gpu.py')
for code in [0, 7]:
    result = subprocess.run([sys.executable, str(launcher), '--', sys.executable, '-c',
        'import os,stat,sys; p=os.environ["TMPDIR"]; print(p,flush=True); '
        'assert stat.S_IMODE(os.stat(p).st_mode)==0o700; '
        f'sys.exit({code})'], text=True, capture_output=True)
    assert result.returncode == code, result.stderr
    assert not Path(result.stdout.strip()).exists(), 'cleanup failed'
with tempfile.TemporaryDirectory() as work:
    parent = Path(work)/'unsafe'
    parent.mkdir(mode=0o777)
    parent.chmod(0o777)
    child = parent/'child'
    child.mkdir(mode=0o700)
    for candidate in [parent, child]:
        env = dict(os.environ, TMPDIR=str(candidate), TMP=str(candidate), TEMP=str(candidate))
        result = subprocess.run([sys.executable, str(launcher), '--', sys.executable, '-c',
                                 'raise SystemExit(0)'], env=env, capture_output=True)
        assert result.returncode != 0, 'untrusted writable ancestry accepted'
print('PASS: Linux private launcher mode, ancestor rejection, cleanup and child failure')
