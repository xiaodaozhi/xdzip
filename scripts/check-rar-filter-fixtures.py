"""Check fixed original-input RAR filters; binaries stay in build, never execute."""
import argparse
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[1]
RESULT = re.compile(r'FILTER_AUTHOR_RESULT\t([^\t]+)\t(29|50)\t(\d+)\t([0-9a-f]{64})\t(\[[0-9, ]+\])')
EXPECTED = {
    'fixed-official591-e8e9': (29, 2048, '6de2e3e6e612b27a70138e34c512a2799c16f709a663583d5d50c36fe3f884f3', [0, 1, 0, 0, 0, 0]),
    'fixed-official591-itanium': (29, 524288, 'd6d8cd0cc8833fda7b5f4cc89ff77bd32b358a3b9e6e53f00e0e095d647cf4cb', [0, 0, 8, 0, 0, 0]),
    'fixed-official723-modern-delta': (50, 524288, 'b8ecd227124f8d48f2b8309969c4ba16a2e9d5eb54b629beed907a8433528f0c', [8, 0, 0, 0]),
    'upstream-arm-independent-original': (50, 90808, 'e68c62b49184ed764f324fb4722481d60e1bf321b722238d95247f391960605c', [2, 0, 0, 4]),
}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to((ROOT / 'build').resolve()):
        raise ValueError('output must be under workspace build/')
    output.mkdir(parents=True, exist_ok=True)
    inputs = output / ('inputs-' + uuid.uuid4().hex)
    report = dict(schema_version=1, status='running', platform=sys.platform,
                  machine=platform.machine(), cases=[],
                  network_policy='Each archive/original response <=128KiB; license <=16KiB. Aggregate is recorded, not limited to128KiB.',
                  limits=['Codec/component original-byte/CRC and actual filter checks, not worker/service/GUI acceptance.',
                          'Modern E8/E8E9 execution is not established by these fixed cases.',
                          'No generated ISA-like or downloaded ELF bytes executed; public GPL binary inputs remain build-only.'])
    def save():
        (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    def run(label, command, environment=None):
        completed = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True,
                                   text=True, encoding='utf-8', errors='replace', timeout=180)
        text = completed.stdout + '\n' + completed.stderr
        for value, replacement in [(str(ROOT), '<workspace>'), (str(Path.home()), '<home>')]:
            text = text.replace(value, replacement)
        (output / (label + '.log')).write_text(text, encoding='utf-8')
        if completed.returncode:
            raise RuntimeError(label + ' failed; see local build log')
        return text
    save()
    try:
        run('prepare-public-arm', [sys.executable, 'crates/xdzip-codec-rar/tests/prepare-public-filter-fixtures.py', '--output', str(inputs)])
        original_meta = json.loads((inputs / 'sources.json').read_text(encoding='utf-8'))
        report['public_sources'] = original_meta
        report['network_download_body_bytes'] = original_meta['archive']['encoded_bytes'] + original_meta['independent_original']['bytes'] + original_meta['independent_original']['license_bytes']
        ordinary = run('fixed-originals', ['cargo', 'test', '-p', 'xdzip-codec-rar', '--locked', 'fixed_official_', '--', '--nocapture', '--test-threads=1'])
        environment = os.environ.copy()
        environment['XDZIP_RAR_FILTER_AUTHOR_FIXTURES'] = str(inputs)
        arm = run('public-arm', ['cargo', 'test', '-p', 'xdzip-codec-rar', '--locked', 'modern_independent_author', '--', '--ignored', '--nocapture', '--test-threads=1'], environment)
        observed = {}
        for label, method, length, digest, counts in RESULT.findall(ordinary + '\n' + arm):
            if label in observed:
                raise ValueError('duplicate result')
            observed[label] = (int(method), int(length), digest, json.loads(counts))
        if observed != EXPECTED:
            raise ValueError('exact known-byte/filter evidence mismatch')
        for label, (method, length, digest, counts) in observed.items():
            report['cases'].append(dict(case=label, status='passed', method=method,
                                        original_bytes=length, original_sha256=digest,
                                        filter_counts=counts,
                                        count_semantics='Successful completed transform runs' if method == 29 else 'Descriptors plus complete successful transform/output and original-byte equality'))
        report['ordinary_tests_passed'] = 3
        report['explicit_tests_passed'] = 1
        report['status'] = 'passed'
        save()
    except Exception as error:
        report['status'] = 'failed'
        report['error'] = type(error).__name__ + ': ' + str(error)
        save()
        raise
    print(json.dumps(dict(status='passed', cases=len(report['cases']), original_bytes=sum(c['original_bytes'] for c in report['cases']))))

if __name__ == '__main__':
    main()
