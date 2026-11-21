#!/usr/bin/env python3
"""Plan a latest-only Drive sync from a checked export and observed inventory."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import re

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('marketing_package', ROOT / 'scripts/package-marketing-assets.py')
package = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package)


def valid_path(name):
    if not isinstance(name, str) or not name or '\\' in name:
        return False
    path = PurePosixPath(name)
    return not path.is_absolute() and path.as_posix() == name and all(part not in ('', '.', '..') for part in path.parts)


def owned_files(previous, platform):
    if previous is None:
        return set()
    source_platform = previous.get('platform')
    if source_platform is None and 'msix_version' in previous and 'baseline' in previous:
        source_platform = 'windows'
    entries = previous.get('assets', previous.get('files'))
    if previous.get('schema_version') != 1 or source_platform != platform or not isinstance(entries, list):
        raise ValueError('Previous manifest has the wrong platform or schema')
    names = set()
    for entry in entries:
        if not isinstance(entry, dict):
            raise ValueError('Invalid previous manifest entry')
        name = entry.get('file')
        if (not valid_path(name) or name in names or name in ('manifest.json', 'SHA256SUMS.txt')
                or not re.fullmatch(r'[a-f0-9]{64}', entry.get('sha256', ''))):
            raise ValueError('Invalid previous manifest entry')
        names.add(name)
    return names | {'manifest.json', 'SHA256SUMS.txt'}


def build_plan(platform, source, inventory, previous=None):
    data, manifest, _ = package.export_files(source, platform)
    if not isinstance(inventory, dict) or not isinstance(inventory.get('files'), list):
        raise ValueError('Inventory must contain a files array')
    remote = {}
    ids = set()
    for entry in inventory['files']:
        if not isinstance(entry, dict):
            raise ValueError('Remote inventory has an invalid entry')
        name, file_id = entry.get('path'), entry.get('id')
        if not valid_path(name) or not isinstance(file_id, str) or not file_id or name in remote or file_id in ids:
            raise ValueError('Remote inventory has an invalid or duplicate path or ID')
        remote[name] = file_id
        ids.add(file_id)
    owned = owned_files(previous, platform)
    collisions = (set(data) & set(remote)) - owned
    if collisions:
        raise ValueError('Existing files are not recorded in the previous manifest: ' + ', '.join(sorted(collisions)))
    # Write the manifest and checksums last, after all data and removal steps succeed.
    metadata = ('manifest.json', 'SHA256SUMS.txt')
    operations = []
    for name in sorted(set(data) - set(metadata)):
        op = {'action': 'update' if name in remote else 'create', 'path': name,
              'sha256': hashlib.sha256(data[name]).hexdigest(), 'bytes': len(data[name])}
        if name in remote:
            op['id'] = remote[name]
        operations.append(op)
    for name in sorted((owned - set(data)) & set(remote)):
        operations.append({'action': 'trash', 'path': name, 'id': remote[name]})
    for name in metadata:
        op = {'action': 'update' if name in remote else 'create', 'path': name,
              'sha256': hashlib.sha256(data[name]).hexdigest(), 'bytes': len(data[name])}
        if name in remote:
            op['id'] = remote[name]
        operations.append(op)
    directories = sorted({str(parent) for name in data for parent in PurePosixPath(name).parents if str(parent) != '.'})
    return {'schema_version': 1, 'platform': platform, 'status': manifest.get('status', 'verified-captures'),
            'captured_languages': manifest.get('captured_languages', []), 'directories': directories,
            'operations': operations}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--platform', choices=package.PLATFORMS, required=True)
    parser.add_argument('--input', type=Path, required=True)
    parser.add_argument('--remote-inventory', type=Path, required=True, help='observed Drive paths and IDs as JSON')
    parser.add_argument('--previous-manifest', type=Path, help='last completed remote manifest, if any')
    args = parser.parse_args()
    inventory = json.loads(args.remote_inventory.read_text(encoding='utf-8'))
    previous = json.loads(args.previous_manifest.read_text(encoding='utf-8')) if args.previous_manifest else None
    print(json.dumps(build_plan(args.platform, args.input, inventory, previous), ensure_ascii=False, indent=2))


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, UnicodeError, json.JSONDecodeError) as error:
        raise SystemExit(str(error)) from error
