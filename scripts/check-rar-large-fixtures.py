"""Check single/17-volume RAR26 streams of the same owned original, without plaintext files."""
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
SPLIT_SHA = '7319c547fdbfaaad63b136e5e305db5f9e73fb19d822990c79cc346508e0bbbd'
ORIGINAL_SHA = 'd1ea9e8a1dd77c140253106db1be9331430622ee141e57a1ccdf31bab1d5a13a'
SIZE = 2 * 1024**3 + 1
RESULT = re.compile(r'RAR_LARGE_RESULT\t26\t(\d+)\t([0-9a-f]{64})\t([0-9a-f]{64})\t(\d+)\t(\d+)\t(\d+)\tcrc_verified')
SPLIT_RESULT = re.compile(r'RAR_LARGE_SPLIT_RESULT\t26\t17\t(\d+)\t([0-9a-f]{64})\t([0-9a-f]{64})\t(\d+)\t(\d+)\t(\d+)\tcrc_verified')


def checked_result(match, archive_sha):
    count, original_sha, actual_archive_sha, work, auxiliary, maximum_chunk = match
    if (int(count), original_sha, actual_archive_sha, int(work)) != (SIZE, ORIGINAL_SHA, archive_sha, SIZE):
        raise ValueError('full original hash/size/work mismatch')
    if not 0 < int(auxiliary) <= 20 * 1024**3 or not 0 < int(maximum_chunk) <= 8192:
        raise ValueError('bounded auxiliary/chunk admission')
    return dict(decoded_bytes=int(count), output_work_bytes=int(work), auxiliary_work=int(auxiliary),
                maximum_output_chunk=int(maximum_chunk), full_crc_verified=True, remaining_prepaid_credit=0)


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
                  scope='Two explicit release-mode component decodes of the same original: single volume and seventeen physical volumes. Incremental full SHA/size/CRC and private prepaid-credit checks; no author executable or plaintext-sized allocation.',
                  limits=['Native worker/runtime high-ratio policy and desktop acceptance are separate.',
                          'Two genuine version26/method1 cases do not establish all historical split/solid/encrypted combinations.',
                          'total_decoded_bytes counts two decodes of the same original, not one4GiB file or two distinct originals.'])

    def save():
        (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')

    save()
    try:
        if report['archive_bytes'] != 1061809 or report['archive_sha256'] != ARCHIVE_SHA:
            raise ValueError('fixture identity')
        split_root = ROOT / 'crates/xdzip-codec-rar/tests/fixtures/legacy26-split-owned'
        split_manifest = json.loads((split_root / 'sources.json').read_text(encoding='utf-8'))
        physical = split_manifest['volumes']
        if len(physical) != 17 or split_manifest['ordered_volumes_sha256'] != SPLIT_SHA:
            raise ValueError('split manifest identity')
        ordered, archive_bytes = hashlib.sha256(), 0
        for item in physical:
            if Path(item['file']).name != item['file']:
                raise ValueError('split basename')
            path = split_root / item['file']
            if path.stat().st_size != item['bytes'] or not 0 < item['bytes'] <= 65536:
                raise ValueError('physical volume size')
            data = path.read_bytes()
            if hashlib.sha256(data).hexdigest() != item['sha256']:
                raise ValueError('physical volume identity')
            ordered.update(data)
            archive_bytes += len(data)
        if archive_bytes != 1063842 or ordered.hexdigest() != SPLIT_SHA:
            raise ValueError('ordered physical volumes identity')
        started = time.monotonic()
        command = ['cargo', 'test', '-p', 'xdzip-codec-rar', '--locked', '--release',
                   'tests::legacy26_owned::official_290_version26_',
                   '--', '--ignored', '--nocapture', '--test-threads=1']
        completed = subprocess.run(command, cwd=ROOT, env=dict(os.environ, CARGO_INCREMENTAL='0'),
                                   capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=300)
        log = completed.stdout + '\n' + completed.stderr
        for value, replacement in [(str(ROOT), '<workspace>'), (str(Path.home()), '<home>')]:
            log = log.replace(value, replacement)
        (output / 'streaming.log').write_text(log, encoding='utf-8')
        if completed.returncode:
            raise RuntimeError('explicit release test failed; see local build log')
        matches, split_matches = RESULT.findall(log), SPLIT_RESULT.findall(log)
        if len(matches) != 1 or len(split_matches) != 1 or 'test result: ok. 2 passed; 0 failed;' not in log:
            raise ValueError('exactly two distinct streaming results required')
        single_result, split_result = checked_result(matches[0], ARCHIVE_SHA), checked_result(split_matches[0], SPLIT_SHA)
        report['cases'] = [
            dict(case=report['case'], status='passed', physical_volume_count=1, original_bytes=SIZE,
                 original_sha256=ORIGINAL_SHA, archive_bytes=report['archive_bytes'], archive_sha256=ARCHIVE_SHA,
                 **single_result),
            dict(case='official-rar290-original-version26-seventeen-volumes', status='passed',
                 physical_volume_count=17, original_bytes=SIZE, original_sha256=ORIGINAL_SHA,
                 archive_bytes=archive_bytes, archive_sha256=SPLIT_SHA,
                 archive_identity_semantics='SHA256 of complete physical bytes in authorized volume order; each volume also independently pinned.',
                 physical_volumes=[{k: item[k] for k in ('file', 'bytes', 'sha256')} for item in physical],
                 **split_result)]
        report.update(status='passed', elapsed_seconds=time.monotonic()-started,
                      **single_result, explicit_tests_passed=2, total_decoded_bytes=2*SIZE)
        save()
    except Exception as error:
        report.update(status='failed', error=type(error).__name__ + ': ' + str(error))
        save()
        raise
    print(json.dumps(dict(status='passed', original_bytes=SIZE, explicit_tests_passed=2,
                          cases=2, total_decoded_bytes=2*SIZE)))


if __name__ == '__main__':
    main()
