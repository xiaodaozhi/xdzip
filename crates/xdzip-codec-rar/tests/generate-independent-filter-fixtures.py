"""Generate owned input recipes and ask fixed official authors to compress them.

No RAR compression encoder is implemented. Generated ISA-like bytes are data;
never execute them. All outputs must be under the workspace build directory.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import subprocess

ROOT = Path(__file__).resolve().parents[3]
AUTHORS = {
    '591': ('0eace28b4c19dfd96ae9ca9b57567de7cde46a1506b7f731cfd700754baed15a', 'https://www.rarlab.com/rar/winrar-x64-591.exe'),
    '723': ('f561764bc3e9ed208744321a89a819b562edeaf06e203c02a06976121fda1991', 'https://www.rarlab.com/rar/winrar-x64-723.exe'),
}

def sha(data):
    return hashlib.sha256(data).hexdigest()

def itanium_original():
    output = bytearray()
    for i in range(32768):
        bundle = 22
        for slot in range(3):
            start = 5 + 41 * slot
            bundle |= 5 << (start + 37)
            bundle |= ((0x4000 - i + slot * 4) & 0xfffff) << (start + 13)
        output.extend(bundle.to_bytes(16, 'little'))
    return bytes(output)

def e8e9_original():
    output = bytearray()
    for i in range(256):
        position = len(output)
        output.append(0xe8 if i % 8 == 0 else 0xe9)
        output.extend((0x10000 - position - 5).to_bytes(4, "little", signed=True))
        output.extend(b"\x90" * 3)
    return bytes(output)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--author-591', required=True, type=Path)
    parser.add_argument('--author-723', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to((ROOT / 'build').resolve()):
        raise ValueError('output must be under workspace build/')
    output.mkdir(parents=True, exist_ok=True)
    if any(output.iterdir()):
        raise ValueError('use an empty output directory; previous evidence is preserved')
    authors = {'591': args.author_591.resolve(), '723': args.author_723.resolve()}
    for release, author in authors.items():
        if sha(author.read_bytes()) != AUTHORS[release][0]:
            raise ValueError('official author identity mismatch')
    recipes = [
        ('e8e9-auto', '591', 29, 'e8e9-auto.bin', e8e9_original(),
         ['-ma4', '-mca-', '-mcd-', '-mcc-', '-mci-', '-mct-'], 1),
        ('itanium', '591', 29, 'ia64-raw.bin', itanium_original(),
         ['-ma4', '-mci+', '-mce-', '-mcd-', '-mca-', '-mcc-', '-mct-'], 2),
        ('modern-delta4', '723', 50, 'modern-delta4.bin',
         bytes(((i // 4) * (i % 4 + 1) + i // 1024) % 256 for i in range(524288)),
         ['-ma5', '-mc4d+', '-mce-'], 0),
    ]
    cases = []
    for label, release, method, original_name, data, switches, required in recipes:
        original = output / original_name
        original.write_bytes(data)
        # RAR4 always has a DOS-shaped mtime field. Keep source metadata fixed;
        # its serialized local calendar can still differ with author timezone.
        os.utime(original, (1700000000, 1700000000))
        archive = output / (label + '.rar')
        command = ['a', '-cfg-', '-m5', '-md4m', '-mt1', '-idq', '-y', '-tsm-',
                   *switches, archive.name, original.name]
        author = authors[release]
        created = subprocess.run([str(author), *command], cwd=output, capture_output=True, timeout=90)
        checked = subprocess.run([str(author), 't', '-cfg-', '-idq', archive.name], cwd=output, capture_output=True, timeout=30)
        decoded = subprocess.run([str(author), 'p', '-cfg-', '-idq', archive.name, original.name], cwd=output, capture_output=True, timeout=30)
        if created.returncode or checked.returncode or decoded.returncode or decoded.stdout != data:
            raise RuntimeError('independent author/readback failed: ' + label)
        record = dict(label=label, release=release, method=method,
                      original=original.name, original_bytes=len(data), original_sha256=sha(data),
                      archive=archive.name, archive_bytes=archive.stat().st_size,
                      archive_sha256=sha(archive.read_bytes()), command=command,
                      author_sha256=AUTHORS[release][0], distribution_url=AUTHORS[release][1],
                      official_t_and_full_original_readback=True,
                      required_filter=required,
                      coverage='must additionally execute candidate tests; author switch alone is not evidence')
        cases.append(record)
        with (output / 'cases.tsv').open('a', encoding='utf-8') as manifest:
            manifest.write('\t'.join([label, str(method), archive.name, record['archive_sha256'],
                                      original.name, record['original_sha256'], str(required)]) + '\n')
    (output / 'sources.json').write_text(json.dumps(dict(
        scope='Owned finite original recipes, independently compressed by fixed official executables. No RAR compression encoder, no generated binary execution.',
        cases=cases), indent=2) + '\n', encoding='utf-8')

if __name__ == '__main__':
    main()
