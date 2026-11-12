#!/usr/bin/env python3
"""Validate a same-run MSIX artifact before extracting the portable control."""
import hashlib
import json
import os
from pathlib import Path
import zipfile

from desktop import verify_msix_identity


def main():
    if os.environ.get('GITHUB_ACTIONS') != 'true' or os.environ.get('RUNNER_ENVIRONMENT') != 'github-hosted':
        raise ValueError('Disposable hosted CI only')
    folder = Path('build/msix-ci-package')
    package = folder / 'xdzip-windows-x64.msix'
    report = json.loads((folder / 'windows-msix.json').read_text(encoding='utf-8'))
    receipt = json.loads((folder / 'package-msix.json').read_text(encoding='utf-8'))
    artifacts = receipt.get('artifacts', [])
    if (receipt.get('status') != 'passed' or receipt.get('source', {}).get('dirty') is not False or
            receipt['source'].get('revision') != os.environ.get('GITHUB_SHA') or len(artifacts) != 1 or
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
    main()
