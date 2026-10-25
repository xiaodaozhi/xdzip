"""Compare modern encrypted candidate files with the fixed official worker.

The public fixture passwords never enter command lines or evidence. Expected
project regression bytes and an official decode are distinct from original
author-input provenance; this report does not claim the latter for old fixtures.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--worker', type=Path, required=True)
    parser.add_argument('--probe', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--authored', type=Path)
    parser.add_argument('--legacy',action='store_true')
    args = parser.parse_args()
    output = args.output.resolve()
    if not any(output.is_relative_to(ROOT / name) for name in ['build', 'dist']):
        raise SystemExit('evidence_output_scope')
    spec = importlib.util.spec_from_file_location('native', ROOT / 'scripts/check-native-codecs.py')
    native = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(native)
    base = ROOT / 'tests/fixtures/native-archives'
    manifest = json.loads((base / 'sources.json').read_text(encoding='utf-8'))
    names = ['test_read_format_rar5_encrypted.rar', 'test_read_format_rar5_encrypted_filenames.rar',
             'test_read_format_rar5_solid_encrypted.rar', 'test_read_format_rar5_solid_encrypted_filenames.rar']
    if args.legacy:
        names += ['test_read_format_rar4_encrypted.rar','test_read_format_rar4_encrypted_filenames.rar']
    results = []
    for name in names:
        path = base / name
        archive = path.read_bytes()
        expected_sha = next(row['sha256'] for row in manifest['fixtures'] if row['file'] == name)
        assert hashlib.sha256(archive).hexdigest() == expected_sha, 'fixture_identity'
        rows, _ = native.run_worker(args.worker, [archive], password='password', format_id=4)
        assert len(rows) == 4, 'metadata_count'
        for row in rows:
            password = 'password2' if name in [names[0],'test_read_format_rar4_encrypted.rar'] and row['name'] == 'd.txt' else 'password'
            env = os.environ.copy()
            env['XDZIP_RAR_PROBE_PASSWORD'] = password
            run = subprocess.run([str(args.probe), 'decode', str(row['index']), str(path)], env=env,
                                 capture_output=True, timeout=60)
            assert run.returncode == 0, (name, row['index'], run.stderr.decode(errors='replace'))
            _, expected = native.run_worker(args.worker, [archive], password=password,
                                             format_id=4, selected=row['index'])
            assert run.stdout == expected, (name, row['index'], 'full_official_bytes')
            assert run.stdout == ('This is from ' + row['name']).encode('utf-8'), 'existing_project_regression_bytes'
            results.append(dict(case=name, entry=row['index'], name=row['name'], bytes=len(run.stdout),
                                sha256=hashlib.sha256(run.stdout).hexdigest(), official_full_bytes=True,
                                independent_original_input=False, status='passed'))
    if args.authored:
        authored = json.loads((args.authored / 'sources.json').read_text(encoding='utf-8'))
        for case in authored['cases']:
            paths = [args.authored / item['file'] for item in case['archives']]
            parts = [path.read_bytes() for path in paths]
            for item, data in zip(case['archives'], parts):
                assert hashlib.sha256(data).hexdigest() == item['sha256'], 'authored_archive_identity'
            password = 'owned-test' if case['encrypted'] else None
            rows, _ = native.run_worker(args.worker, parts, password=password, format_id=4)
            assert len(rows) == len(case['inputs']), 'authored_metadata_count'
            for row in rows:
                env = os.environ.copy()
                env.pop('XDZIP_RAR_PROBE_PASSWORD', None)
                if password is not None:
                    env['XDZIP_RAR_PROBE_PASSWORD'] = password
                run = subprocess.run([str(args.probe), 'decode', str(row['index']), *map(str, paths)],
                                     env=env, capture_output=True, timeout=60)
                assert run.returncode == 0, (case['case'], row['index'], run.stderr.decode(errors='replace'))
                _, reference = native.run_worker(args.worker, parts, password=password, format_id=4, selected=row['index'])
                expected = next(item for item in case['inputs'] if item['name'] == row['name'])
                content = (paths[0].parent / row['name']).read_bytes()
                assert hashlib.sha256(content).hexdigest() == expected['sha256'], 'owned_input_identity'
                assert run.stdout == reference == content, (case['case'], row['index'], 'full_known_bytes')
                results.append(dict(case=case['case'], entry=row['index'], name=row['name'], bytes=len(content),
                                    sha256=hashlib.sha256(run.stdout).hexdigest(), physical_volumes=len(parts),
                                    first_visible_packed=row['packed'], official_full_bytes=True,
                                    independent_original_input=True, status='passed'))
    report = dict(scope='Modern encrypted container/payload candidate; not product worker integration or full RAR migration.',
                  probe_sha256=hashlib.sha256(args.probe.read_bytes()).hexdigest(),
                  reference_sha256=hashlib.sha256(args.worker.read_bytes()).hexdigest(),
                  fixture_manifest_sha256=hashlib.sha256((base / 'sources.json').read_bytes()).hexdigest(),
                  results=results)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(dict(cases=len(results), bytes=sum(row['bytes'] for row in results),
                          independent_original_input_cases=sum(row['independent_original_input'] for row in results))))


if __name__ == '__main__':
    main()
