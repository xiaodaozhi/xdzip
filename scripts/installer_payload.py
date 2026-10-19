"""Freeze the exact audited file set before an installer copies any bytes."""
from deployment_audit import digest, within


def verify_payload(root,audit):
    expected={f['path']:f['sha256'] for f in audit['files']}
    actual={}
    for path in root.rglob('*'):
        if path.is_symlink() or not within(path,root):raise ValueError('Installer payload symlink rejected')
        if path.is_file():actual[path.relative_to(root).as_posix()]=digest(path)
        elif not path.is_dir():raise ValueError('Installer special file rejected')
    if actual!=expected:raise ValueError('Audited payload changed or contains unrecorded files')
