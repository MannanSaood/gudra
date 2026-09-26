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
    def identities(paths, *, resolve_symlinks=False):
        entries = []
        for path in paths:
            candidate = path.resolve(strict=True) if resolve_symlinks else path
            if (path.is_symlink() and not resolve_symlinks) or not candidate.is_file():
                raise SystemExit('FAIL: artifacts/tools must be regular files')
            entries.append({'name': path.name, 'sha256': digest(candidate),
                            'bytes': candidate.stat().st_size})
        if len({e['name'] for e in entries}) != len(entries):
            raise SystemExit('FAIL: duplicate artifact/tool basenames')
        return entries
    artifacts = identities(args.artifact)
    tools = identities(args.tool, resolve_symlinks=True)
    lock_packages = tomllib.loads((ROOT/'Cargo.lock').read_text())['package']
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--locked', '--format-version', '1'], cwd=ROOT, text=True
    ))
    metadata_by_key = {(p['name'], p['version']): p for p in metadata['packages']}

    def component(package):
        info = metadata_by_key.get((package['name'], package['version']), {})
        result = {
            'type': 'library',
            'bom-ref': f"pkg:cargo/{package['name']}@{package['version']}",
            'name': package['name'],
            'version': package['version'],
            'purl': f"pkg:cargo/{package['name']}@{package['version']}",
        }
        if package.get('checksum'):
            result['hashes'] = [{'alg': 'SHA-256', 'content': package['checksum']}]
        if info.get('license'):
            result['licenses'] = [{'expression': info['license']}]
        source = info.get('source') or 'vendored-or-workspace-path'
        result['properties'] = [{'name': 'cargo:source', 'value': source}]
        return result

    components = [component(p) for p in lock_packages]
    root_component = next(c for c in components if c['name'] == 'gudra')
    sbom = {
        'bomFormat': 'CycloneDX',
        'specVersion': '1.5',
        'version': 1,
        'metadata': {'component': root_component},
        'components': [c for c in components if c is not root_component],
    }
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output/'sbom.cdx.json').write_text(json.dumps(sbom, indent=2)+'\n')
    manifest = {'source_revision':revision,'dirty':False,'artifacts':artifacts,'tools':tools,
                'cargo_lock_sha256':digest(ROOT/'Cargo.lock'),
                'vendor_manifest_sha256':digest(ROOT/'vendor/checksums.json'),
                'sbom_sha256':digest(args.output/'sbom.cdx.json'),
                'signed':False,'release_authorization':False,
                'sbom_scope':'complete locked Rust graph and vendored sources; external native tools recorded separately; CUDA toolkit and driver are prerequisites and are not redistributed'}
    (args.output/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    (args.output/'SHA256SUMS').write_text(''.join(f"{e['sha256']}  {e['name']}\n" for e in artifacts))
    print('Unsigned evidence generated; protected workflow attestation remains required.')

if __name__ == '__main__':
    main()
