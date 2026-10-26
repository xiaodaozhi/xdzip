"""Compare the Rust candidate with an explicitly supplied independent worker.

Build the rar-probe example and run the export_authored_vectors ignored test
first. This host tool reads only pinned/project-owned fixtures, never reference
implementation source. Evidence stays under the worktree's build or dist tree.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--worker', type=Path, required=True)
    parser.add_argument('--probe', type=Path, required=True)
    parser.add_argument('--authored', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if not any(output.is_relative_to(ROOT / name) for name in ['build', 'dist']):
        raise SystemExit('evidence_output_scope')
    spec = importlib.util.spec_from_file_location('native_check', ROOT / 'scripts/check-native-codecs.py')
    native = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(native)
    fixed = ROOT / 'tests/fixtures/native-archives'
    manifest = json.loads((fixed / 'sources.json').read_text(encoding='utf-8'))
    for item in manifest['fixtures']:
        assert hashlib.sha256((fixed / item['file']).read_bytes()).hexdigest() == item['sha256'], 'fixture_identity'
    results = []

    def check(label, paths, content=None):
        parts = [path.read_bytes() for path in paths]
        official, _ = native.run_worker(args.worker, parts, format_id=4)
        run = subprocess.run([str(args.probe), 'metadata', *map(str, paths)], capture_output=True, check=True, timeout=45)
        rust = []
        for line in run.stdout.decode().splitlines():
            fields = line.split('\t')
            rust.append(dict(id=int(fields[0]), method=int(fields[1]), size=int(fields[2]), packed=int(fields[3]), encrypted=fields[4] == '1', directory=fields[5] == '1', link=fields[6] == '1', raw=fields[7], name=bytes.fromhex(fields[8]).decode('utf-8'), size_known=fields[9] == '1'))
        assert len(rust) == len(official), (label, 'count')
        for a, b in zip(rust, official):
            assert a['id'] == b['index'] + 1, (label, 'id')
            if label == 'authored-three-part':
                assert a['size_known'] and a['size'] == 6 and b['size'] == 0x7fffffff7fffffff, (label, 'resolved_size_projection')
                assert (a['method'], a['packed']) == (b['method'], b['packed']), (label, 'numeric')
            elif label == 'authored-all-unknown':
                assert not a['size_known'] and a['size'] == 0 and b['size'] == 0x7fffffff7fffffff, (label, 'unknown_size_projection')
                assert (a['method'], a['packed']) == (b['method'], b['packed']), (label, 'numeric')
            else:
                assert a['size_known'], (label, 'known_size')
                assert (a['method'], a['size'], a['packed']) == (b['method'], b['size'], b['packed']), (label, 'numeric')
            assert a['name'].rstrip('/') == b['name'].replace('\\', '/').rstrip('/'), (label, 'name')
            assert a['encrypted'] == bool(b['flags'] & 2), (label, 'encrypted')
            assert a['directory'] == bool(b['flags'] & 1), (label, 'directory')
            assert a['link'] == bool(b['flags'] & 4), (label, 'link')
            if content is not None:
                i = b['index']
                own = subprocess.run([str(args.probe), 'decode', str(i), *map(str, paths)], capture_output=True, check=True, timeout=45).stdout
                assert own == content[i], (label, 'candidate_known_bytes', i)
                # The preserved native publishes an unknown sentinel for this
                # first fragment. The current host correctly refuses that DTO
                # before payload; keep its guard and compare candidate bytes
                # with the independent mathematical input instead.
                if label != 'authored-three-part':
                    _, decoded = native.run_worker(args.worker, parts, format_id=4, selected=i)
                    assert decoded == content[i], (label, 'reference_known_bytes', i)
        results.append(dict(case=label, files=len(rust), known_bytes=content is not None,
                            decoded_bytes=sum(map(len, content)) if content is not None else 0,
                            independent_worker_payload=(content is not None and label != 'authored-three-part'),
                            size_projection=(
                                'candidate resolved final known size; native exposes first-header unknown sentinel'
                                if label == 'authored-three-part' else
                                'candidate explicitly retains unknown with zero numeric placeholder; native exposes internal sentinel'
                                if label == 'authored-all-unknown' else 'equal'),
                            sha256=[hashlib.sha256(part).hexdigest() for part in parts], status='passed'))

    for label, expected in [('authored-old-store', [b'abc', b'defg', b'']), ('authored-modern-store', [b'abc', b'defg', b'']), ('authored-unicode', [b'unicode-content'])]:
        check(label, [args.authored / (label + '.rar')], expected)
    check('authored-three-part', [args.authored / f'authored-split.part{i:02}.rar' for i in range(1, 4)], [b'abcdef'])
    check('authored-all-unknown', [args.authored / 'authored-all-unknown.rar'])
    check('fixed-rar7-store', [fixed / 'rar7-store-small.rar'], [b'RAR newest algorithm verification sample'])
    for name in ['test_read_format_rar.rar', 'test_read_format_rar_compress_normal.rar', 'test_read_format_rar5_compressed.rar', 'test_read_format_rar5_multiple_files_solid.rar', 'rar-7.23-rar5-compatible.rar']:
        check(name, [fixed / name])
    check('fixed-rar5-solid-four-part', [fixed / f'test_read_format_rar5_multiarchive_solid.part{i:02}.rar' for i in range(1, 5)])
    report = dict(scope='Container metadata and Store only. No compressed Rust RAR algorithm, crypto or production-route claim.',
                  probe_sha256=hashlib.sha256(args.probe.read_bytes()).hexdigest(), reference_sha256=hashlib.sha256(args.worker.read_bytes()).hexdigest(),
                  fixture_manifest_sha256=hashlib.sha256((fixed / 'sources.json').read_bytes()).hexdigest(), results=results)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(f'{len(results)} independently compared container cases passed')


if __name__ == '__main__':
    main()
