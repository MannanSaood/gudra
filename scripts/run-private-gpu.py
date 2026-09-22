#!/usr/bin/env python3
"""Run a trusted GPU command with private JIT temporary storage (Linux only)."""
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile


def main():
    if sys.platform != "linux":
        raise SystemExit("GPU validation requires Linux; private temp ACLs are not verified on this platform.")
    command = sys.argv[1:]
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        raise SystemExit("usage: run-private-gpu.py -- <trusted command> [arguments]")
    parent = Path(tempfile.gettempdir()).resolve()
    for ancestor in (parent, *parent.parents):
        info = ancestor.stat()
        mode = stat.S_IMODE(info.st_mode)
        protected_shared = info.st_uid == 0 and bool(info.st_mode & stat.S_ISVTX)
        if info.st_uid not in (0, os.getuid()) or (mode & 0o022 and not protected_shared):
            raise SystemExit("Temporary-directory ancestors must prevent other users from replacing entries.")
    # mkdtemp atomically creates a mode-0700 directory. A directory boundary
    # protects both the visible bytecode and the as-yet-uncreated cubin name.
    with tempfile.TemporaryDirectory(prefix="gudra-jit-", dir=parent) as directory:
        info = Path(directory).stat()
        if info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700:
            raise SystemExit("JIT temp directory must be owned by this user with mode 0700.")
        env = os.environ.copy()
        env.update(TMPDIR=directory, TMP=directory, TEMP=directory)
        # No shell and no cross-user artifact imports. Environment is fixed
        # before process startup, rather than mutated by a multithreaded library.
        return subprocess.run(command, env=env, check=False).returncode


if __name__ == "__main__":
    sys.exit(main())
