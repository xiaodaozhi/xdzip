"""Use the fixed official author on owned input, without an archive encoder."""
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
    admitted = next(row for row in source['fixtures'] if row['file'] == 'rar-7.23-rar5-compatible.rar')['tool']
    assert hashlib.sha256(args.rar.read_bytes()).hexdigest() == admitted['sha256'], 'author_identity'
    output.mkdir(parents=True, exist_ok=True)
    cases = []
    settings = [
        ('empty-store-crc', 0, False, False, False, None, [0]),
        ('plain-store-blake', 0, False, False, True, None, [65]),
        ('empty-encrypted-store', 0, True, False, False, None, [0]),
        ('encrypted-store-crc', 0, True, False, False, None, [17]),
        ('encrypted-store-blake', 0, True, False, True, None, [513]),
        ('encrypted-lz-blake-headers', 5, True, True, True, None, [65537]),
        ('encrypted-solid-blake-headers', 5, True, True, True, None, [32769, 65537]),
        ('encrypted-split-store-blake', 0, True, True, True, '4093b', [20000]),
        ('encrypted-split-solid-lz-blake', 5, True, True, True, '4093b', [32769, 65537]),
    ]
    for label, method, encrypted, headers, blake, volume, lengths in settings:
        directory = output / label
        directory.mkdir(exist_ok=True)
        inputs = []
        for index, length in enumerate(lengths):
            name = f'owned-{index}.bin'
            content = bytes((i * 29 + (i >> 8) * 3 + index * 11 + 7) & 255 for i in range(length))
            formula = f'byte[i]=(29*i+3*(i>>8)+{index}*11+7) mod256'
            if label == 'encrypted-split-solid-lz-blake':
                state = 0x9e3779b9 ^ index
                mixed = bytearray(content)
                for i in range(length):
                    state ^= (state << 13) & 0xffffffff
                    state ^= state >> 17
                    state ^= (state << 5) & 0xffffffff
                    if i % 1024 < 512:
                        mixed[i] = state & 255
                content = bytes(mixed)
                formula += '; for all i advance u32 xorshift(13,17,5), seed=0x9e3779b9 XOR index; when i mod1024<512 replace byte with state low8'
            (directory / name).write_bytes(content)
            inputs.append(dict(name=name, bytes=length, sha256=hashlib.sha256(content).hexdigest(),
                               formula=formula))
        switches = ['a', '-cfg-', f'-m{method}', '-md4m', '-mt1', '-idq', '-y', '-tsm-']
        if len(inputs) > 1:
            switches.append('-s')
        if blake:
            switches.append('-htb')
        if encrypted:
            switches.append('-hpowned-test' if headers else '-powned-test')
        if volume:
            switches.append('-v' + volume)
        switches += ['owned.rar', *[row['name'] for row in inputs]]
        run = subprocess.run([str(args.rar), *switches], cwd=directory, capture_output=True, timeout=120)
        assert run.returncode == 0, (label, 'author_creation')
        archives = sorted(directory.glob('*.rar'))
        assert archives, 'created_archive'
        test = subprocess.run([str(args.rar), 't', '-cfg-', '-idq', '-y', '-powned-test', archives[0].name],
                              cwd=directory, capture_output=True, timeout=120)
        assert test.returncode == 0, (label, 'official_full_test')
        # Public test password deliberately omitted from command/provenance.
        cases.append(dict(case=label, inputs=inputs, encrypted=encrypted, headers=headers, blake=blake,
                          solid=len(inputs) > 1, method=method, volume_argument=volume,
                          archives=[dict(file=f'{label}/{p.name}', sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in archives],
                          official_test_exit=test.returncode))
    report = dict(scope='Owned inputs, independent official RAR author; no RAR encoder implementation.', tool=admitted, cases=cases)
    (output / 'sources.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(dict(cases=len(cases), volumes=sum(len(row['archives']) for row in cases))))


if __name__ == '__main__':
    main()
