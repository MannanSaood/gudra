#!/usr/bin/env python3
"""Separate-UID test. Run only as root in an explicitly disposable Linux VM."""
import argparse
import os
from pathlib import Path
import pwd
import subprocess
import sys
import tempfile

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--disposable', action='store_true', required=True)
    parser.add_argument('--victim', required=True)
    parser.add_argument('--attacker', required=True)
    args = parser.parse_args()
    if sys.platform != 'linux' or os.geteuid() != 0:
        raise SystemExit('FAIL: requires root in a disposable Linux environment')
    victim, attacker = (pwd.getpwnam(n) for n in (args.victim, args.attacker))
    if 0 in (victim.pw_uid, attacker.pw_uid) or victim.pw_uid == attacker.pw_uid:
        raise SystemExit('FAIL: require two distinct non-root accounts')
    root = Path(__file__).resolve().parents[1]
    with tempfile.TemporaryDirectory(prefix='gudra-uid-test-') as work:
        work = Path(work)
        work.chmod(0o755)
        module = root / 'vendor/cutile-compiler/src/private_temp.rs'
        (work/'main.rs').write_text(module.read_text() + '''
fn main() {
    let dir = PrivateTemp::new(&format!("{:x}", std::process::id())).unwrap();
    dir.write(&dir.path().join("input.bc"), b"harmless fixture").unwrap();
    println!("{}", dir.path().display());
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
}
''')
        subprocess.run(['rustc', '--edition=2021', str(work/'main.rs'), '-o', str(work/'probe')], check=True)
        subprocess.run(['rustc', '--edition=2021', '--test', str(module), '-o', str(work/'unit')], check=True)
        subprocess.run([str(work/'unit')], check=True)
        process = subprocess.Popen([str(work/'probe')], user=victim.pw_uid, group=victim.pw_gid,
                                   extra_groups=[], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        path = Path(process.stdout.readline().strip())
        if path.parent != Path('/tmp') or not path.name.startswith('cutile-private-'):
            process.kill()
            raise SystemExit('FAIL: unexpected helper path')
        try:
            attack = '''import os,sys
p=sys.argv[1]
operations=[lambda: os.listdir(p), lambda: open(p+'/input.bc','rb'),
lambda: open(p+'/output.cubin','wb'), lambda: os.unlink(p+'/input.bc'),
lambda: os.rename(p,p+'-replaced'), lambda: os.symlink('/etc/passwd',p+'/output.cubin')]
for op in operations:
 try: op()
 except PermissionError: continue
 raise SystemExit('FAIL: cross-UID operation was allowed')
'''
            subprocess.run([sys.executable, '-c', attack, str(path)], user=attacker.pw_uid,
                           group=attacker.pw_gid, extra_groups=[], check=True)
        finally:
            process.communicate('\n', timeout=10)
        if process.returncode or path.exists():
            raise SystemExit('FAIL: normal cleanup')
    print('PASS: private-temp unit tests and separate-UID access/replacement rejection')

if __name__ == '__main__':
    main()
