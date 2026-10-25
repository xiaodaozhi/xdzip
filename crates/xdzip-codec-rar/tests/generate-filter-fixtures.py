"""Create owned branch-pattern data with the pinned official RAR author.

This invokes an independent author as a black box. It contains no RAR encoder,
decoder or reference implementation source; generated binaries are never run.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rar', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if not any(output.is_relative_to(ROOT / name) for name in ['build', 'dist']):
        raise SystemExit('fixture_output_scope')
    source = json.loads((ROOT / 'tests/fixtures/native-archives/sources.json').read_text(encoding='utf-8'))
    admitted = next(item for item in source['fixtures'] if item['file'] == 'rar-7.23-rar5-compatible.rar')['tool']
    digest = hashlib.sha256(args.rar.read_bytes()).hexdigest()
    assert digest == admitted['sha256'], 'official_author_identity'
    output.mkdir(parents=True, exist_ok=True)
    cases = []
    for name, mixed in [('e8-only', False), ('e8e9', True)]:
        content = bytearray()
        for i in range(65536):
            position = i * 8
            target = (i * 97) % 524288
            content.append(0xe9 if mixed and i % 2 else 0xe8)
            content.extend((target - position - 5).to_bytes(4, 'little', signed=True))
            content.extend([0x90, 0x50, 0x58])
        input_path = output / (name + '.bin')
        archive = output / (name + '.rar')
        input_path.write_bytes(content)
        switches = ['a', '-cfg-', '-m5', '-md4m', '-mt1', '-mce+', '-mcd-', '-mcl+', '-idq', '-y', '-tsm-', archive.name, input_path.name]
        run = subprocess.run([str(args.rar), *switches], cwd=output, capture_output=True, timeout=120)
        assert run.returncode == 0, 'official_author_creation'
        checked = subprocess.run([str(args.rar), 't', '-cfg-', '-idq', '-y', archive.name], cwd=output, capture_output=True, timeout=60)
        assert checked.returncode == 0, 'official_author_full_test'
        cases.append(dict(file=archive.name, sha256=hashlib.sha256(archive.read_bytes()).hexdigest(), input=input_path.name,
                          input_bytes=len(content), input_sha256=hashlib.sha256(content).hexdigest(), command=[args.rar.name, *switches],
                          author_test_exit=checked.returncode, expected_filter='E8/E8E9 must be measured in candidate; command alone is not coverage'))
    report = dict(scope='Owned branch bytes authored by official RAR; no RAR compression implementation or algorithm1 claim',
                  tool=admitted, tool_sha256_verified=digest, fixtures=cases)
    (output / 'sources.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(f'{len(cases)} owned branch fixtures authored and tested')


if __name__ == '__main__':
    main()
