#!/usr/bin/env python3
"""Check desktop CLI resource layout and actual macOS code/resource sealing."""
import argparse
import importlib.util
import json
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import cli

ROOT = Path(__file__).resolve().parents[1]


class DesktopBundleTests(unittest.TestCase):
    def setUp(self):
        parent = ROOT/'build/test-tmp'
        parent.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(prefix='cli-bundle-', dir=parent)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.root_patch = patch.object(cli, 'ROOT', self.root)
        self.root_patch.start()
        self.addCleanup(self.root_patch.stop)

    def bundle(self, system):
        directory = self.root/'build'/system/'cli'
        (directory/'bin').mkdir(parents=True)
        suffix = '.exe' if system == 'Windows' else ''
        for name in ('xdzip', 'xdzip-launcher', 'xdzip-native-codec'):
            (directory/'bin'/(name+suffix)).write_bytes(b'owned program')
        output = (self.root/'build'/system/'Owned.app/Contents/Helpers/cli'
                  if system == 'Darwin' else self.root/'build'/system/'desktop/cli')
        output.mkdir(parents=True)
        (output/'xdzip-cli-context.json').write_bytes(b'obsolete generated marker')
        args = argparse.Namespace(build_dir=directory, output=output)
        with patch.object(cli.platform, 'system', return_value=system):
            cli.bundle(args)
            cli.bundle(args)
        return output

    def test_repeated_bundles_place_metadata_in_the_platform_resource_location(self):
        for system, relative in (('Darwin', '../../MacOS/xdzip'),
                                 ('Windows', '../xdzip.exe'), ('Linux', '../xdzip')):
            with self.subTest(system=system):
                output = self.bundle(system)
                marker = (output.parent.parent/'Resources/xdzip-cli-context.json'
                          if system == 'Darwin' else output/'xdzip-cli-context.json')
                self.assertEqual(json.loads(marker.read_bytes()), {
                    'schema_version': 1, 'kind': 'desktop', 'desktop_relative': relative})
                if system == 'Darwin':
                    self.assertFalse((output/marker.name).exists())
                    self.assertEqual(len(list(output.iterdir())), 3)

    @unittest.skipUnless(sys.platform == 'darwin' and shutil.which('xcrun') and shutil.which('codesign'),
                         'Native macOS compiler and signing tools required')
    def test_native_signing_accepts_resource_context_and_seals_its_bytes(self):
        output = self.bundle('Darwin')
        app = output.parents[2]
        executable = app/'Contents/MacOS/xdzip'
        executable.parent.mkdir()
        subprocess.run(['xcrun', 'clang', '-x', 'c', '-o', executable, '-'],
                       input=b'int main(void) { return 0; }\n', capture_output=True, check=True)
        for path in output.iterdir():
            shutil.copy2(executable, path)
        (app/'Contents/Info.plist').write_bytes(plistlib.dumps({
            'CFBundleIdentifier': 'org.xdzip.owned-cli-seal', 'CFBundleExecutable': 'xdzip',
            'CFBundlePackageType': 'APPL', 'CFBundleVersion': '1'}))
        marker = app/'Contents/Resources/xdzip-cli-context.json'
        obsolete = output/marker.name
        shutil.copyfile(marker, obsolete)
        rejected = subprocess.run(['codesign', '--dryrun', '--force', '--sign', '-',
                                   '--timestamp=none', executable], capture_output=True)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn(marker.name.encode(), rejected.stderr)
        obsolete.unlink()
        spec = importlib.util.spec_from_file_location('owned_cli_signer', ROOT/'scripts/build-macos-dmg.py')
        builder = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(builder)

        def checked(arguments):
            result = subprocess.run(list(map(str, arguments)), capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            return result.stdout

        builder.checked = checked
        builder.sign_local(app)
        marker.write_bytes(marker.read_bytes()+b' ')
        rejected = subprocess.run(['codesign', '--verify', '--deep', '--strict', app], capture_output=True)
        self.assertNotEqual(rejected.returncode, 0)


if __name__ == '__main__':
    unittest.main()
