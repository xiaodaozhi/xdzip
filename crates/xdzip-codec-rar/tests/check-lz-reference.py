"""Actual RAR5 candidate/reference comparison; no RAR encoder or decoder source."""
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
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--authored', type=Path)
    parser.add_argument('--filters', type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    if not any(output.is_relative_to(ROOT / name) for name in ['build', 'dist']):
        raise SystemExit('evidence_output_scope')
    spec = importlib.util.spec_from_file_location('native', ROOT / 'scripts/check-native-codecs.py')
    native = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(native)
    base = ROOT / 'tests/fixtures/native-archives'
    manifest = json.loads((base / 'sources.json').read_text(encoding='utf-8'))
    for item in manifest['fixtures']:
        assert hashlib.sha256((base / item['file']).read_bytes()).hexdigest() == item['sha256'], 'fixture_identity'
    modern = next(item for item in manifest['fixtures'] if item['file'] == 'rar-7.23-rar5-compatible.rar')

    authored_inputs = {}

    def expected_input(label, name):
        if label in authored_inputs:
            return authored_inputs[label]
        if label == 'authored-store-in-solid.rar' and name.startswith('store-'):
            return b'Store never enters the LZ dictionary'
        if label == 'rar-7.23-rar5-compatible.rar':
            return (modern['input']['text'] * modern['input']['repeat']).encode('utf-8')
        if label == 'test_read_format_rar5_compressed.rar':
            length, modifier = 1200, 0
        elif label in ['test_read_format_rar5_multiple_files_solid.rar', 'authored-store-in-solid.rar']:
            length, modifier = 4096, int(name[4])
        else:
            return None
        return b''.join(max(0, k * k - 3 * k + 1 + modifier).to_bytes(4, 'little') for k in range(1, length // 4 + 1))

    groups = [(name, [base / name]) for name in ['test_read_format_rar5_compressed.rar', 'test_read_format_rar5_multiple_files_solid.rar', 'rar-7.23-rar5-compatible.rar']]
    groups.append(('test_read_format_rar5_multiarchive_solid.part01.rar', [base / f'test_read_format_rar5_multiarchive_solid.part{i:02}.rar' for i in range(1, 5)]))
    if args.authored:
        groups.append(('authored-store-in-solid.rar', [args.authored / 'authored-store-in-solid.rar']))
    if args.filters:
        source = json.loads((args.filters / 'sources.json').read_text(encoding='utf-8'))
        for item in source['fixtures']:
            path = args.filters / item['file']
            content = (args.filters / item['input']).read_bytes()
            assert hashlib.sha256(path.read_bytes()).hexdigest() == item['sha256'], 'authored_archive_identity'
            assert hashlib.sha256(content).hexdigest() == item['input_sha256'], 'authored_input_identity'
            authored_inputs[item['file']] = content
            groups.append((item['file'], [path]))
    results = []
    for label, paths in groups:
        parts = [path.read_bytes() for path in paths]
        rows, _ = native.run_worker(args.worker, parts, format_id=4)
        for row in rows:
            run = subprocess.run([str(args.probe), 'decode', str(row['index']), *map(str, paths)], capture_output=True, timeout=60)
            assert run.returncode == 0, (label, row['index'], run.stderr.decode(errors='replace'))
            _, expected = native.run_worker(args.worker, parts, format_id=4, selected=row['index'])
            assert run.stdout == expected, (label, 'reference_bytes')
            mathematical = expected_input(label, row['name'])
            if mathematical is not None:
                assert run.stdout == mathematical, (label, 'independent_known_bytes')
            results.append(dict(case=label, entry=row['index'], name=row['name'], bytes=len(expected), sha256=hashlib.sha256(run.stdout).hexdigest(), independent_known_bytes=mathematical is not None, status='passed'))
    report = dict(scope='Actual Rust algorithm0 and solid/authorized-volume decoding. Algorithm1 independent compressed success remains issue43; not a product route.',
                  probe_sha256=hashlib.sha256(args.probe.read_bytes()).hexdigest(), reference_sha256=hashlib.sha256(args.worker.read_bytes()).hexdigest(),
                  fixture_manifest_sha256=hashlib.sha256((base / 'sources.json').read_bytes()).hexdigest(), results=results)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(dict(cases=len(results), bytes=sum(item['bytes'] for item in results), known_bytes_cases=sum(item['independent_known_bytes'] for item in results))))


if __name__ == '__main__':
    main()
