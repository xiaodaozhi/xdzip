#!/usr/bin/env python3
"""Validate a same-run MSIX artifact before extracting the portable control."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import zipfile

from desktop import verify_msix_identity


RECHECK_FILES = frozenset({
    '.github/workflows/windows-msix-recheck.yml',
    '.github/workflows/windows-msix-coexistence.yml',
    '.github/workflows/windows-msix.yml',
    'docs/engineering/ci-policy.md',
    'docs/engineering/shell-actions.md',
    'scripts/check-windows-msix-install.ps1',
    'scripts/msix-origin-probe.py',
    'scripts/test-msix-origin-probe.py',
    'scripts/prepare-msix-upgrade.py',
    'scripts/test-msix-upgrade.py',
    'scripts/windows-msix-activation.cs',
    'scripts/extract-msix-ci.py',
    'scripts/test-msix-ci.py',
    'scripts/prepare-msix-coexistence.py',
    'scripts/check-windows-msix-coexistence.ps1',
})


def validate_source_revision(source, current):
    if not re.fullmatch(r'[0-9a-f]{40}', source or '') or not re.fullmatch(r'[0-9a-f]{40}', current or ''):
        raise ValueError('Invalid source revision')
    if source == current:
        return
    ancestor = subprocess.run(['git', 'merge-base', '--is-ancestor', source, current], check=False)
    if ancestor.returncode:
        raise ValueError('Test source does not descend from the package source')
    changed = subprocess.run(['git', 'diff', '--name-only', '--no-renames', '-z', source, current],
                             check=True, capture_output=True).stdout.decode('utf-8').split('\0')
    if any(name not in RECHECK_FILES for name in changed if name):
        raise ValueError('Package inputs changed since the package was built')


def main(source_revision=None):
    if os.environ.get('GITHUB_ACTIONS') != 'true' or os.environ.get('RUNNER_ENVIRONMENT') != 'github-hosted':
        raise ValueError('Disposable hosted CI only')
    folder = Path('build/msix-ci-package')
    package = folder / 'xdzip-windows-x64.msix'
    report = json.loads((folder / 'windows-msix.json').read_text(encoding='utf-8'))
    receipt = json.loads((folder / 'package-msix.json').read_text(encoding='utf-8'))
    source_revision = source_revision or os.environ.get('GITHUB_SHA')
    validate_source_revision(source_revision, os.environ.get('GITHUB_SHA'))
    artifacts = receipt.get('artifacts', [])
    if (receipt.get('status') != 'passed' or receipt.get('source', {}).get('dirty') is not False or
            receipt['source'].get('revision') != source_revision or len(artifacts) != 1 or
            artifacts[0].get('file') != package.name or report.get('status') != 'passed' or
            report.get('name') != 'xdzip.MsixCI' or report.get('publisher') != 'CN=xdzip MSIX CI' or
            report.get('signed') is not False or report.get('modern_context_menu') is not False):
        raise ValueError('Unexpected same-run package receipt')
    with package.open('rb') as source:
        digest = hashlib.file_digest(source, 'sha256').hexdigest()
    if digest != artifacts[0].get('sha256') or digest != report.get('sha256'):
        raise ValueError('Package bytes differ from the same-run receipts')
    verify_msix_identity(package, report)
    output = folder / 'windows'
    if output.exists():
        raise ValueError('Portable control already exists')
    with zipfile.ZipFile(package) as archive:
        archive.extractall(output)
    if not (output / 'xdzip.exe').is_file():
        raise ValueError('Portable control executable is missing')
    print('Verified same-run unsigned MSIX and prepared portable control')


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-revision', help='Validated package source for a test-only recheck')
    main(parser.parse_args().source_revision)
