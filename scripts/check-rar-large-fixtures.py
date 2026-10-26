"""Run the pinned original-input RAR26 fixture without materializing plaintext."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE_SHA = 'e2b067cfcc7aff4f8ad9e8b9cef3755dec65d9109bbcccf328c679a8546d2148'
ORIGINAL_SHA = 'd1ea9e8a1dd77c140253106db1be9331430622ee141e57a1ccdf31bab1d5a13a'
SIZE = 2 * 1024**3 + 1
RESULT = re.compile(r'RAR_LARGE_RESULT\t26\t(\d+)\t([0-9a-f]{64})\t([0-9a-f]{64})\t(\d+)\t(\d+)\t(\d+)\tcrc_verified')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to((ROOT / 'build').resolve()):
        raise ValueError('output must stay below workspace build/')
    output.mkdir(parents=True, exist_ok=True)
    fixture = ROOT / 'crates/xdzip-codec-rar/tests/fixtures/legacy26-owned/large26.rar'
    report = dict(schema_version=1, status='running', platform=sys.platform, machine=platform.machine(),
                  case='official-rar290-original-version26', original_bytes=SIZE, original_sha256=ORIGINAL_SHA,
                  archive_bytes=fixture.stat().st_size, archive_sha256=hashlib.sha256(fixture.read_bytes()).hexdigest(),
                  scope='Explicit release-mode component decode; incremental hash, full size/CRC and private prepaid-credit checks. No author executable, plaintext file or plaintext-sized allocation.',
                  limits=['Native worker/runtime high-ratio policy and desktop acceptance are separate.',
                          'One genuine single-file version26/method1 archive does not establish all large-file or split combinations.'])

    def save():
        (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')

    save()
    try:
        if report['archive_bytes'] != 1061809 or report['archive_sha256'] != ARCHIVE_SHA:
            raise ValueError('fixture identity')
        started = time.monotonic()
        command = ['cargo', 'test', '-p', 'xdzip-codec-rar', '--locked', '--release',
                   'tests::legacy26_owned::official_290_version26_large_streaming_sha_crc_and_bounded_sink',
                   '--', '--exact', '--ignored', '--nocapture', '--test-threads=1']
        completed = subprocess.run(command, cwd=ROOT, env=dict(os.environ, CARGO_INCREMENTAL='0'),
                                   capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=300)
        log = completed.stdout + '\n' + completed.stderr
        for value, replacement in [(str(ROOT), '<workspace>'), (str(Path.home()), '<home>')]:
            log = log.replace(value, replacement)
        (output / 'streaming.log').write_text(log, encoding='utf-8')
        if completed.returncode:
            raise RuntimeError('explicit release test failed; see local build log')
        matches = RESULT.findall(log)
        if len(matches) != 1:
            raise ValueError('exactly one streaming result required')
        count, original_sha, archive_sha, work, auxiliary, maximum_chunk = matches[0]
        if (int(count), original_sha, archive_sha, int(work)) != (SIZE, ORIGINAL_SHA, ARCHIVE_SHA, SIZE):
            raise ValueError('full original hash/size/work mismatch')
        if not 0 < int(auxiliary) <= 20 * 1024**3 or not 0 < int(maximum_chunk) <= 8192:
            raise ValueError('bounded auxiliary/chunk admission')
        report.update(status='passed', elapsed_seconds=time.monotonic()-started,
                      decoded_bytes=int(count), output_work_bytes=int(work), auxiliary_work=int(auxiliary),
                      maximum_output_chunk=int(maximum_chunk), full_crc_verified=True,
                      remaining_prepaid_credit=0, explicit_tests_passed=1)
        save()
    except Exception as error:
        report.update(status='failed', error=type(error).__name__ + ': ' + str(error))
        save()
        raise
    print(json.dumps(dict(status='passed', original_bytes=SIZE, explicit_tests_passed=1)))


if __name__ == '__main__':
    main()
