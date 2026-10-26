"""Collect bounded public acceptance records, excluding every signed candidate."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil

TOP_RECORDS = {
    'prepared.json', 'production-update-prepared.json', 'production-signing.json',
    'production-signing-result.json', 'production-execution-verification.json',
    'production-trust-verification.json',
    'result.json', 'verification.json', 'helper-request.json', 'helper-result.json',
    'normal-quit.json', 'installer-windows.json', 'interrupted-handoff-verification.json',
    'ordinary-recovery-verification.json', 'replacement-terminal.json',
    'native-replacement-error.json', 'replacement-flags-restored.json',
    'replacement-failure-verification.json', 'replacement-failure-recovery.json',
    'quit-first-intent.json', 'ordinary-quit-verification.json', 'quit-wait-failure.json',
    'association-before-preparation.json', 'association-original.json',
    'association-baseline.json', 'association-after-update.json',
    'association-after-restore.json', 'association-after-unregister.json',
}
SESSION_RECORD = re.compile(
    r'(?:(?:200|201)-(?:state|started|exit-barrier)|quit-[a-z-]+|'
    r'password-[a-z-]+|installer-copy-[a-z-]+|replacement-[a-z-]+)\.json\Z')
PER_FILE_LIMIT = 16 * 1024 * 1024
TOTAL_LIMIT = 96 * 1024 * 1024


def collect(directory, output):
    if directory.is_symlink() or output.is_symlink():
        raise ValueError('Evidence paths must not be symlinks')
    directory = directory.resolve()
    output = output.resolve()
    if directory == output or directory in output.parents or output in directory.parents:
        raise ValueError('Evidence output must be separate from fixture input')
    output.mkdir(parents=True, exist_ok=False)
    candidates = []
    if directory.is_dir():
        candidates.extend(p for p in directory.iterdir()
                          if p.name in TOP_RECORDS or p.suffix == '.log' or
                          (p.name.startswith('session-') and SESSION_RECORD.fullmatch(p.name[8:])))
        for folder in ('failed-install-recovery', 'session'):
            parent = directory / folder
            if parent.is_symlink():
                raise ValueError('Evidence subdirectory must not be a symlink')
            if parent.is_dir():
                candidates.extend(p for p in parent.iterdir()
                                  if (folder == 'failed-install-recovery' and
                                      (p.name in {'result.json', 'helper-request.json', 'helper-result.json'} or p.suffix == '.log')) or
                                  (folder == 'session' and SESSION_RECORD.fullmatch(p.name)))
    records = []
    total = 0
    if len(candidates) > 200:
        raise ValueError('Too many evidence records')
    for path in sorted(candidates):
        if path.is_symlink() or not path.is_file():
            raise ValueError('Evidence member must be an ordinary file')
        if path.stat().st_size > PER_FILE_LIMIT:
            raise ValueError('Evidence member exceeds the bounded log allowance')
        content = path.read_bytes()
        total += len(content)
        if total > TOTAL_LIMIT:
            raise ValueError('Evidence exceeds total allowance')
        if path.suffix == '.json':
            value = json.loads(content)
            # No root release manifest, signature envelope or signing metadata.
            if isinstance(value, dict) and (
                    {'assets', 'tag', 'repository'} <= value.keys() or
                    {'algorithm', 'signature'} <= value.keys() or
                    {'ed_signature', 'filename'} <= value.keys()):
                raise ValueError('Signed candidate appeared in an evidence-only slot')
        relative = path.relative_to(directory).as_posix()
        target = output / relative.replace('/', '--')
        shutil.copyfile(path, target)
        records.append({'source': relative, 'filename': target.name, 'bytes': len(content),
                        'sha256': hashlib.sha256(content).hexdigest()})
    receipt = {'schema_version': 1, 'status': 'collected', 'fixture_directory_present': directory.is_dir(),
               'acceptance_passed': False, 'note': 'Collection alone does not establish acceptance.',
               'signed_candidates_included': False, 'applications_included': False,
               'bytes': total, 'records': records}
    (output / 'evidence-index.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = collect(args.directory, args.output)
    print('Collected public evidence records: ' + str(len(result['records'])))


if __name__ == '__main__':
    main()
