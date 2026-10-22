"""Freeze the exact audited file set before an installer copies any bytes."""
import os
from deployment_audit import digest, within


HISTORICAL_PRODUCT_REVISION = '286fe9239a1d2d0d5a599f59953c502fb37a58a0'


def historical_qa_revision(revision):
    if os.environ.get('CI') != 'true' or revision != HISTORICAL_PRODUCT_REVISION:
        raise ValueError('Historical helper omission requires disposable CI and the fixed full revision')


def historical_qa_identity(revision, identity):
    historical_qa_revision(revision)
    if (identity.get('revision') != revision[:12] or identity.get('version') != '0.1.0-dev.1'
            or type(identity.get('build')) is not int or identity['build'] != 1
            or identity.get('dirty') is not False):
        raise ValueError('Historical helper omission requires the pinned clean product identity')
    return {'revision': revision, 'update_helper_in_historical_source': False,
            'scope': 'CI-only frozen historical product, never a current product package'}


def verify_payload(root,audit):
    expected={f['path']:f['sha256'] for f in audit['files']}
    actual={}
    for path in root.rglob('*'):
        if path.is_symlink() or not within(path,root):raise ValueError('Installer payload symlink rejected')
        if path.is_file():actual[path.relative_to(root).as_posix()]=digest(path)
        elif not path.is_dir():raise ValueError('Installer special file rejected')
    if actual!=expected:raise ValueError('Audited payload changed or contains unrecorded files')
