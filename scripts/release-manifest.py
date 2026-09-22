#!/usr/bin/env python3
"""Produce unsigned artifact/dependency evidence; does not authorize publication."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--artifact', type=Path, action='append', required=True)
    parser.add_argument('--tool', type=Path, action='append', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if subprocess.check_output(['git','status','--porcelain'], cwd=ROOT).strip():
        raise SystemExit('FAIL: require a clean approved checkout')
    revision = subprocess.check_output(['git','rev-parse','HEAD'], cwd=ROOT, text=True).strip()
    def identities(paths):
        entries = []
        for path in paths:
            if path.is_symlink() or not path.is_file():
                raise SystemExit('FAIL: artifacts/tools must be regular files')
            entries.append({'name': path.name, 'sha256': digest(path), 'bytes': path.stat().st_size})
        if len({e['name'] for e in entries}) != len(entries):
            raise SystemExit('FAIL: duplicate artifact/tool basenames')
        return entries
    artifacts, tools = identities(args.artifact), identities(args.tool)
    packages = tomllib.loads((ROOT/'Cargo.lock').read_text())['package']
    sbom = {'bomFormat':'CycloneDX','specVersion':'1.5','version':1,
            'components':[{'type':'library','name':p['name'],'version':p['version'],
                           'purl':f"pkg:cargo/{p['name']}@{p['version']}",
                           **({'hashes':[{'alg':'SHA-256','content':p['checksum']}]} if p.get('checksum') else {})}
                          for p in packages]}
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output/'sbom.cdx.json').write_text(json.dumps(sbom, indent=2)+'\n')
    manifest = {'source_revision':revision,'dirty':False,'artifacts':artifacts,'tools':tools,
                'cargo_lock_sha256':digest(ROOT/'Cargo.lock'),
                'vendor_manifest_sha256':digest(ROOT/'vendor/checksums.json'),
                'sbom_sha256':digest(args.output/'sbom.cdx.json'),
                'signed':False,'release_authorization':False,
                'sbom_scope':'locked Rust packages; native tools listed separately; OS/driver inventory required'}
    (args.output/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    (args.output/'SHA256SUMS').write_text(''.join(f"{e['sha256']}  {e['name']}\n" for e in artifacts))
    print('Unsigned evidence generated; protected signing and native inventory remain required.')

if __name__ == '__main__':
    main()
