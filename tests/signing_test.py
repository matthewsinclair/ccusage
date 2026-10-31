import hashlib
import importlib.machinery
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import sys
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]


def load(name, filename):
    loader = importlib.machinery.SourceFileLoader(name, str(ROOT / 'tools' / filename))
    module = importlib.util.module_from_spec(importlib.util.spec_from_loader(name, loader))
    loader.exec_module(module)
    return module


sign = load('sign_test', 'sign-macos')
finalize = load('finalize_test', 'finalize-release')


class SigningTests(unittest.TestCase):
    def archives(self, source, tag='v0.1.4'):
        source.mkdir()
        for target in sign.TARGETS:
            with tarfile.open(source / f'ccusage-{tag}-{target}.tar.gz', 'w:gz') as tar:
                for name in sign.FILES:
                    data = (target if name == 'ccusage' else name).encode()
                    member = tarfile.TarInfo(name)
                    member.size = len(data)
                    tar.addfile(member, io.BytesIO(data))

    def test_signed_mac_archives_preserve_linux_and_bind_checksums(self):
        with tempfile.TemporaryDirectory() as temp:
            source, output = Path(temp) / 'in', Path(temp) / 'out'
            self.archives(source)
            def run(*args):
                if args[0].endswith('lipo'):
                    return 'arm64' if Path(args[-1]).read_text().startswith('aarch64') else 'x86_64'
                Path(args[-1]).write_bytes(Path(args[-1]).read_bytes() + b'-signed')
                return ''
            with patch.object(sign, 'run', side_effect=run), patch.object(sign, 'verify', return_value='designated => stable'), patch.dict(sign.os.environ, {}, clear=True):
                sign.prepare('v0.1.4', source, output)
            entries = finalize.validate_checksums(output, 'v0.1.4')
            manifest = json.loads((output / 'MACOS-SIGNING.json').read_text())
            for target in sign.TARGETS:
                name = f'ccusage-v0.1.4-{target}.tar.gz'
                if 'linux' in target:
                    self.assertEqual((source / name).read_bytes(), (output / name).read_bytes())
                else:
                    self.assertEqual(manifest['archives'][name]['sha256'], entries[name])
            (output / next(iter(entries))).write_bytes(b'tampered')
            with self.assertRaises(ValueError):
                finalize.validate_checksums(output, 'v0.1.4')

    def test_archive_path_traversal_and_symlinks_are_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for name, kind in [('../escape', tarfile.REGTYPE), ('ccusage', tarfile.SYMTYPE)]:
                archive = root / 'bad.tar.gz'
                with tarfile.open(archive, 'w:gz') as tar:
                    item = tarfile.TarInfo(name)
                    item.type = kind
                    item.linkname = '/tmp/elsewhere'
                    tar.addfile(item)
                with self.assertRaises(ValueError):
                    sign.extract(archive, root)

    def test_failed_signing_never_creates_publication_manifest(self):
        with tempfile.TemporaryDirectory() as temp:
            source, output = Path(temp) / 'in', Path(temp) / 'out'
            self.archives(source)
            with patch.object(sign, 'run', side_effect=['arm64', subprocess.CalledProcessError(1, 'codesign')]):
                with self.assertRaises(subprocess.CalledProcessError):
                    sign.prepare('v0.1.4', source, output)
            self.assertFalse((output / 'MACOS-SIGNING.json').exists())
            self.assertFalse((output / 'SHA256SUMS').exists())

    def test_missing_timestamp_or_unstable_requirement_rejected(self):
        for details in ['runtime\ndesignated => stable', 'runtime\nTimestamp=now\ndesignated => cdhash HASH']:
            with patch.object(sign, 'run'), patch.object(sign.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, '', details)):
                with self.assertRaises(ValueError):
                    sign.verify(Path('fixture'))

    def test_signature_requirement_is_read_from_stdout(self):
        result = subprocess.CompletedProcess([], 0, 'designated => identifier "stable"\n', 'Timestamp=now\nflags=runtime\n')
        with patch.object(sign, 'run'), patch.object(sign.subprocess, 'run', return_value=result):
            self.assertEqual(sign.verify(Path('fixture')), 'designated => identifier "stable"')

    @unittest.skipUnless(sys.platform == 'darwin', 'requires macOS requirement parser')
    def test_real_macos_parser_accepts_verification_requirement(self):
        # Obtain the actual -R argument used by verification, then have macOS
        # parse it. Missing '=' is interpreted as a filename by both tools.
        with patch.object(sign, 'run') as command, patch.object(sign.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, 'designated => stable\n', 'Timestamp=now\nruntime\n')):
            sign.verify(Path('fixture'))
        args = command.call_args.args
        requirement = args[args.index('-R') + 1]
        with tempfile.TemporaryDirectory() as temp:
            subprocess.run(['/usr/bin/csreq', '-r', requirement, '-b', str(Path(temp) / 'requirement')], check=True, capture_output=True)

    def test_draft_is_not_published_when_signing_fails(self):
        calls = []
        def run(args, **kwargs):
            calls.append(args)
            if args[:3] == ['gh', 'release', 'download']:
                Path(args[args.index('--dir') + 1]).mkdir(exist_ok=True)
            return subprocess.CompletedProcess(args, 0)
        with patch.object(finalize.sys, 'platform', 'darwin'), patch.object(finalize, 'command', return_value='true'), patch.object(finalize.subprocess, 'run', side_effect=run), patch.object(finalize, 'validate_checksums', return_value={}), patch.object(finalize.sign, 'prepare', side_effect=ValueError('signing failed')):
            with self.assertRaises(ValueError):
                finalize.main('v0.1.4')
        self.assertFalse(any(args[:3] in (['gh', 'release', 'edit'], ['gh', 'release', 'upload']) for args in calls))

    def test_wrong_manifest_identity_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'MACOS-SIGNING.json').write_text(json.dumps({'tag':'v0.1.4','identifier':'wrong','team_id':sign.TEAM}))
            with self.assertRaises(ValueError):
                finalize.verify_manifest(root, 'v0.1.4', {})


if __name__ == '__main__':
    unittest.main()
