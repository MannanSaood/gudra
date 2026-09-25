#!/usr/bin/env python3
"""Capture final GPU evidence, with fault injection only on disposable workers."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--arch-experimental', action='store_true')
    parser.add_argument('--disposable-fault-tests', action='store_true')
    args = parser.parse_args()
    if sys.platform != 'linux':
        raise SystemExit('FAIL: Linux GPU host required')
    dirty = subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True)
    if dirty.strip():
        raise SystemExit('FAIL: candidate must have a clean working tree')
    evidence = {'revision': subprocess.check_output(['git','rev-parse','HEAD'], cwd=ROOT, text=True).strip(),
                'dirty': False, 'lane': 'arch-experimental' if args.arch_experimental else 'pinned',
                'started_utc': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()), 'commands': [],
                'lock_sha256': hashlib.sha256((ROOT/'Cargo.lock').read_bytes()).hexdigest(),
                'disposable_fault_tests': args.disposable_fault_tests,
                'scope': 'normal execution plus deterministic ownership seams; sticky device fault only when explicitly enabled'}
    destination = ROOT/'target/gpu-candidate-evidence.json'
    destination.parent.mkdir(exist_ok=True)
    def run(command):
        result = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        output = result.stdout.replace(str(ROOT), '<repo>').replace(str(Path.home()), '<home>')
        print(output, end='', flush=True)
        evidence['commands'].append({'command': command, 'exit_code': result.returncode, 'output': output})
        destination.write_text(json.dumps(evidence, indent=2)+'\n')
        if result.returncode:
            raise SystemExit(result.returncode)
    for command in [['rustc','-Vv'], ['cargo','-V'], ['nvcc','--version'],
                    [os.environ.get('CUTILE_TILEIRAS_PATH','tileiras'),'--version'],
                    ['nvidia-smi','--query-gpu=name,driver_version,compute_cap','--format=csv,noheader'],
                    ['compute-sanitizer','--version'], ['cargo','tree','--locked','--features','gpu']]:
        run(command)
    if not args.arch_experimental:
        run(['bash','scripts/check-gpu-env.sh'])
    run(['cargo','test','--locked','-p','cutile-compiler','--lib','private_compilation_lifecycle','--','--test-threads=1'])
    run(['cargo','test','--locked','-p','cuda-async','--lib','fault_policy::tests','--','--test-threads=1'])
    run(['cargo','test','--locked','-p','cuda-async','--lib','device_future::release_tests','--','--test-threads=1'])
    run(['cargo','check','--locked','--features','gpu','--all-targets'])
    run(['cargo','clippy','--locked','--features','gpu','--all-targets','--','-D','warnings'])
    run(['cargo','doc','--locked','--features','gpu','--no-deps'])
    run(['python3','scripts/check-ui.py','--gpu'])
    run(['cargo','test','--locked','--features','gpu','--doc'])
    os.environ['CUDA_ASYNC_SPIN_BUDGET_US'] = '0'
    for _ in range(5):
        run(['cargo','test','--locked','--features','gpu','--lib','--test','gpu_jacobi','--','--test-threads=1'])
    run(['cargo','test','--locked','-p','cuda-async','--test','drop_in_flight','--','--test-threads=1'])
    if args.disposable_fault_tests:
        run(['cargo','test','--locked','-p','cuda-async','--test','device_fault','--','--test-threads=1'])
    run(['python3','scripts/check-gpu-sanitizers.py'])
    evidence['normal_path_suite'] = 'pass'
    evidence['release_decision'] = 'not established by this runner; administrative controls are separate'
    destination.write_text(json.dumps(evidence, indent=2)+'\n')
    print('Normal-path suite passed. Review target/gpu-candidate-evidence.json privately before sharing.')

if __name__ == '__main__':
    main()
