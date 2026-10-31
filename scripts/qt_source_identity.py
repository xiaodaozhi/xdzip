#!/usr/bin/env python3
"""Validate pinned Qt archive markers and any required Git object relationship."""
import base64
import binascii
import hashlib
from pathlib import Path, PurePosixPath
import re

from deployment_audit import within

ROOT = Path(__file__).resolve().parents[1]
OBJECT_ID = re.compile(r'[0-9a-f]{40}')


def commit_object_path(item, root=None):
    """Locate an owned, bounded fixed proof input without following a proof symlink."""
    root = (root or ROOT).resolve()
    name = item.get('commit_object')
    if not isinstance(name, str) or '\\' in name or ':' in name:
        raise ValueError('Invalid Git commit object path')
    relative = PurePosixPath(name)
    base = PurePosixPath('scripts/qt-source-identity')
    if relative.is_absolute() or '..' in relative.parts or not relative.is_relative_to(base):
        raise ValueError('Git commit object path escapes proof inputs')
    path = root.joinpath(*relative.parts)
    parents = [root.joinpath(*relative.parts[:index]) for index in range(1, len(relative.parts) + 1)]
    if any(part.is_symlink() for part in parents) or not within(path, root) or not within(path, root / base) or not path.is_file():
        raise ValueError('Git commit object path escapes proof inputs or is missing')
    return path


def source_identity(item, root=None):
    """Bind a marker to a pinned commit; only a verified commit may name a tree."""
    commit = item.get('source_commit')
    kind = item.get('archive_marker_type', 'commit')
    marker = item.get('archive_marker', commit)
    if not isinstance(commit, str) or not OBJECT_ID.fullmatch(commit):
        raise ValueError('Invalid pinned source commit')
    if not isinstance(marker, str) or not OBJECT_ID.fullmatch(marker):
        raise ValueError('Invalid pinned source archive marker')
    if kind == 'commit':
        if marker != commit or 'commit_object' in item:
            raise ValueError('Inconsistent source commit marker')
        verified = False
    elif kind == 'tree':
        proof = commit_object_path(item, root)
        if proof.stat().st_size > 16384:
            raise ValueError('Git commit object proof is too large')
        encoded = proof.read_bytes()
        try:
            payload = base64.b64decode(b''.join(encoded.split()), validate=True)
        except (ValueError, binascii.Error) as error:
            raise ValueError('Invalid Git commit object proof') from error
        digest = hashlib.sha1(b'commit ' + str(len(payload)).encode('ascii') + b'\0' + payload).hexdigest()
        headers, separator, _ = payload.partition(b'\n\n')
        lines = headers.split(b'\n')
        tree_headers = [line for line in lines if line.startswith(b'tree ')]
        expected_tree = b'tree ' + marker.encode('ascii')
        if digest != commit or not separator or not lines or lines[0] != expected_tree or tree_headers != [expected_tree]:
            raise ValueError('Pinned Git commit and archive tree proof mismatch')
        verified = True
    else:
        raise ValueError('Unknown source archive marker type')
    return {'source_commit': commit, 'archive_marker': marker, 'archive_marker_type': kind,
            'commit_tree_verified': verified}
