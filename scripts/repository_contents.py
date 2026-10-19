"""Check durable-source boundaries against the Git index, not ignore patterns."""
from pathlib import PurePosixPath
import subprocess

GENERATED_PREFIXES = ('docs/html/', 'design/assets/preview/', 'design/html/screenshots/')
RETIRED_REPORTS = {
    'docs/qa/private-test-readiness.md', 'docs/qa/wp12-closeout-actions.md',
    'docs/planning/preparation-record.md', 'docs/engineering/toolchain-candidates.json',
    'docs/engineering/support-policy-proposal.md',
}
MAX_BLOB_BYTES = 5 * 1024 * 1024


def violations(root):
    records = subprocess.check_output(['git', 'ls-files', '--stage', '-z'], cwd=root).split(b'\0')
    files = []
    failures = []
    for record in records:
        if not record:
            continue
        metadata, raw_name = record.split(b'\t', 1)
        _, oid, stage = metadata.decode().split()
        name = raw_name.decode()
        if stage != '0':
            failures.append(f'{name}: unresolved index entry')
            continue
        parts = PurePosixPath(name).parts
        if any(p in {'build', 'target', 'dist', '.cache'} for p in parts[:-1]):
            failures.append(f'{name}: generated directories must not be tracked')
        elif name.startswith(GENERATED_PREFIXES) or name in RETIRED_REPORTS:
            failures.append(f'{name}: generated content or retired task report belongs in artifacts/issues')
        elif name.startswith('docs/qa/') and not name.endswith('.md'):
            failures.append(f'{name}: QA execution output must not be tracked')
        files.append((name, oid))
    if files:
        sizes = subprocess.check_output(['git', 'cat-file', '--batch-check=%(objectsize)'],
                                        input=''.join(oid+'\n' for _, oid in files).encode(), cwd=root)
        for (name, _), size in zip(files, sizes.splitlines(), strict=True):
            if int(size) > MAX_BLOB_BYTES:
                failures.append(f'{name}: blob exceeds 5 MiB; review durable-input need before changing the limit')
    return failures
