#!/usr/bin/env python3
"""Reconstruct only audited runtime files for a disposable installer test."""
import hashlib
import json
import os
from pathlib import Path
import shutil

from installer_payload import verify_payload


def main():
    if os.environ.get('GITHUB_ACTIONS') != 'true' or os.environ.get('RUNNER_ENVIRONMENT') != 'github-hosted':
        raise ValueError('Disposable hosted CI only')
    folder = Path('build/msix-ci-package')
    audit_path = Path('build/msix-ci-evidence/msix-ci-package/deployment-audit.json')
    report = json.loads((folder / 'windows-msix.json').read_text(encoding='utf-8'))
    receipt = json.loads((folder / 'package-msix.json').read_text(encoding='utf-8'))
    audit_bytes = audit_path.read_bytes()
    if hashlib.sha256(audit_bytes).hexdigest() != report.get('audit_sha256'):
        raise ValueError('Runtime audit differs from the package producer')
    audit = json.loads(audit_bytes)
    source_revision = receipt.get('source', {}).get('revision', '')
    if (len(source_revision) != 40 or audit.get('status') != 'passed' or
            audit.get('platform') != 'windows' or audit.get('mode') != 'deployed' or
            audit.get('identity', {}).get('revision') != source_revision[:12] or
            audit['identity'].get('dirty') is not False):
        raise ValueError('Runtime audit identity differs from the package')
    source = (folder / 'windows').resolve()
    output = (folder / 'installer-runtime').resolve()
    if output.exists():
        raise ValueError('Installer runtime already exists')
    output.mkdir()
    for record in audit['files']:
        name = record['path']
        if not isinstance(name, str) or '\\' in name:
            raise ValueError('Invalid audited runtime path')
        src, dst = (source / name).resolve(), (output / name).resolve()
        if not src.is_relative_to(source) or not dst.is_relative_to(output) or not src.is_file():
            raise ValueError('Audited runtime file is missing or outside the root')
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src, dst)
    verify_payload(output, audit)
    print('Reconstructed and verified the exact audited installer payload')


if __name__ == '__main__':
    main()
