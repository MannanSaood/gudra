#!/usr/bin/env python3
"""Harmless installer tamper tests; no download or execution."""
import hashlib
import importlib.util
import io
from pathlib import Path
import tarfile
import unittest

spec = importlib.util.spec_from_file_location('installer', Path(__file__).with_name('install-security-tools.py'))
installer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(installer)

class InstallerTests(unittest.TestCase):
    def archive(self, names):
        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode='w:gz') as archive:
            for name in names:
                member = tarfile.TarInfo(name)
                member.size = 7
                archive.addfile(member, io.BytesIO(b'fixture'))
        return stream.getvalue()

    def test_verified_control_and_modified_download(self):
        payload = self.archive(['package/tool'])
        digest = hashlib.sha256(payload).hexdigest()
        self.assertEqual(installer.verified_binary(payload, 'tool', digest), b'fixture')
        with self.assertRaisesRegex(ValueError, 'Checksum mismatch'):
            installer.verified_binary(payload + b'tamper', 'tool', digest)

    def test_missing_and_duplicate_executable_rejected(self):
        for names in [[], ['one/tool', 'two/tool']]:
            payload = self.archive(names)
            with self.assertRaisesRegex(ValueError, 'Ambiguous'):
                installer.verified_binary(payload, 'tool', hashlib.sha256(payload).hexdigest())

if __name__ == '__main__':
    unittest.main()
