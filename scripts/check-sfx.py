#!/usr/bin/env python3
"""Create and run owned SFX archives without an installed xdzip or Qt runtime."""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import struct
import zipfile


def run(args, *, expected=0, env=None):
    process = subprocess.run([str(arg) for arg in args], capture_output=True, timeout=90, env=env)
    if process.returncode != expected:
        raise AssertionError((process.returncode, process.stdout.decode('utf-8', 'replace'), process.stderr.decode('utf-8', 'replace')))
    return process


def contents(root):
    return {path.relative_to(root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in root.rglob('*') if path.is_file()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli', type=Path, required=True)
    parser.add_argument('--stub', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert os.name == 'nt', 'SFX execution verification requires Windows x64'
    output = args.output.resolve()
    root = Path(__file__).resolve().parents[1]
    assert output.is_relative_to(root / 'build') and output != root / 'build'
    assert not output.exists(), 'Use a fresh evidence directory'
    output.mkdir(parents=True)
    sid = next(csv.reader(run(['whoami', '/user', '/fo', 'csv', '/nh']).stdout.decode().splitlines()))[1]
    run(['icacls', output, '/inheritance:r', '/grant:r', '*' + sid + ':(OI)(CI)F'])
    source = output / 'source'
    (source / '中文' / 'empty').mkdir(parents=True)
    (source / '中文' / 'data.bin').write_bytes(bytes(range(256)) * 6000)
    (source / 'zero').write_bytes(b'')
    (source / 'must-not-run.cmd').write_text('@echo UNSAFE>"%~dp0EXECUTED"\n', encoding='ascii')
    expected = {'source/' + name: digest for name, digest in contents(source).items()}
    root_member = output / 'root-member.txt'
    root_member.write_bytes(b'archive member')
    expected[root_member.name] = hashlib.sha256(root_member.read_bytes()).hexdigest()
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith(('XDZIP_', 'QT_', 'QML', 'PYTHONPATH'))}
    environment['PATH'] = str(Path(os.environ['SystemRoot']) / 'System32')
    checks = []
    creator = output / 'isolated-creator'
    creator.mkdir()
    cli_only = creator / 'xdzip.exe'
    shutil.copy2(args.cli.resolve(), cli_only)
    refused_creation = output / 'missing-stub.sfx.exe'
    missing = json.loads(run([cli_only, 'create', refused_creation, source, '--json'], expected=1, env=environment).stdout)
    assert missing['error_code'] == 'sfx_stub_missing' and not refused_creation.exists()
    (creator / 'xdzip-sfx.exe').write_bytes(b'MZ' + bytes(510))
    invalid = json.loads(run([cli_only, 'create', refused_creation, source, '--json'], expected=1, env=environment).stdout)
    assert invalid['error_code'] == 'sfx_stub_invalid' and not refused_creation.exists()
    checks.append('missing-or-invalid-fixed-location-stub-does-not-publish')
    for level in ['store', 'normal']:
        image = output / (level + '.sfx.exe')
        value = json.loads(run([args.cli.resolve(), 'create', image, source, root_member,
                                '--level', level, '--json']).stdout)
        assert value['status'] == 'succeeded', value
        isolated = output / ('isolated-' + level)
        isolated.mkdir()
        executable = isolated / 'archive.exe'
        shutil.copy2(image, executable)
        assert len(list(isolated.iterdir())) == 1
        assert json.loads(run([executable, '--test'], env=environment).stdout)['status'] == 'succeeded'
        target = output / ('extracted-' + level)
        target.mkdir()
        result = json.loads(run([executable, '--extract-to', target, '--conflict', 'skip'], env=environment).stdout)
        assert result['status'] == 'succeeded' and result['committed_files'] == 4, result
        assert Path(result['target']) == target, result
        assert contents(target) == expected
        assert (target / 'source/中文/empty').is_dir()
        assert not (target / 'source/EXECUTED').exists()
        shared = output / ('shared-' + level)
        shared.mkdir()
        run(['icacls', shared, '/grant:r', '*S-1-1-0:(OI)(CI)(M)'])
        isolated_result = json.loads(run([executable, '--extract-to', shared], env=environment).stdout)
        actual = Path(isolated_result['target'])
        assert isolated_result['status'] == 'succeeded' and actual.parent == shared, isolated_result
        assert contents(actual) == expected
        # Test the ordinary-file policies on a member outside any renamed tree.
        (target / root_member.name).write_bytes(b'old destination')
        run([executable, '--extract-to', target, '--conflict', 'skip'], env=environment)
        assert (target / root_member.name).read_bytes() == b'old destination'
        run([executable, '--extract-to', target, '--conflict', 'rename'], env=environment)
        assert (target / root_member.name).read_bytes() == b'old destination'
        assert (target / 'root-member (1).txt').read_bytes() == root_member.read_bytes()
        run([executable, '--extract-to', target, '--conflict', 'overwrite'], env=environment)
        assert (target / root_member.name).read_bytes() == root_member.read_bytes()
        damaged = isolated / 'damaged.exe'
        data = bytearray(executable.read_bytes())
        data[-100] ^= 1
        damaged.write_bytes(data)
        assert json.loads(run([damaged, '--test'], expected=1, env=environment).stdout)['error_code'] == 'integrity_error'
        refused = output / ('refused-' + level)
        refused.mkdir()
        run([damaged, '--extract-to', refused], expected=1, env=environment)
        assert not list(refused.iterdir())
        # A consistent envelope cannot turn damaged ZIP content into a successful full check.
        with zipfile.ZipFile(executable) as archive:
            entry = next(item for item in archive.infolist() if item.filename.endswith('data.bin'))
            changed = bytearray(executable.read_bytes())
            name_length, extra_length = struct.unpack_from('<HH', changed, entry.header_offset + 26)
            changed[entry.header_offset + 30 + name_length + extra_length + 10] ^= 1
        changed[-32:] = hashlib.sha256(changed[:-64]).digest()
        damaged.write_bytes(changed)
        checked = json.loads(run([damaged, '--test'], expected=1, env=environment).stdout)
        assert checked['status'] == 'failed'
        run([damaged, '--extract-to', refused], expected=1, env=environment)
        assert not any(path.name == 'data.bin' for path in refused.rglob('*'))
        for option in [['--encrypt'], ['--volume-size', '1MiB']]:
            rejected = output / (level + '-rejected.sfx.exe')
            run([args.cli.resolve(), 'create', rejected, source, '--json', *option], expected=2)
            assert not rejected.exists()
        checks.append(level + ':standalone-validation-and-extraction-conflicts-damage-options-no-autorun')
    imports = run(['dumpbin', '/dependents', args.stub.resolve()]).stdout.decode('utf-8', 'replace')
    assert not any(name.lower() in imports.lower() for name in ['Qt6', 'vcruntime140', 'msvcp140', 'python', 'xdzip-native-codec'])
    checks.append('stub-has-only-system-runtime-imports')
    receipt = {'status': 'passed', 'cli_sha256': hashlib.sha256(args.cli.read_bytes()).hexdigest(),
               'stub_sha256': hashlib.sha256(args.stub.read_bytes()).hexdigest(), 'checks': checks}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    print('SFX Store/Deflate: standalone execution, conflicts, corruption, option rejection and no-autorun passed')


if __name__ == '__main__':
    main()
