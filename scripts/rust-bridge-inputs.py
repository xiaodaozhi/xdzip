"""List local package directories for a native Cargo bridge build."""

import argparse
import json
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cargo', required=True)
    parser.add_argument('--target', required=True)
    parser.add_argument('--package', required=True)
    args = parser.parse_args()
    metadata = json.loads(subprocess.check_output([
        args.cargo, 'metadata', '--locked', '--format-version', '1',
        '--filter-platform', args.target,
    ]))
    packages = {package['id']: package for package in metadata['packages']}
    nodes = {node['id']: node for node in metadata['resolve']['nodes']}
    pending = [key for key in metadata['workspace_members'] if packages[key]['name'] == args.package]
    if len(pending) != 1:
        parser.error(f'expected one workspace package named {args.package}')

    visited = set()
    directories = set()
    while pending:
        key = pending.pop()
        if key in visited:
            continue
        visited.add(key)
        if packages[key]['source'] is None:
            directories.add(Path(packages[key]['manifest_path']).parent.as_posix())
        pending.extend(dep['pkg'] for dep in nodes[key]['deps']
                       if any(kind['kind'] in (None, 'build') for kind in dep['dep_kinds']))
    print(';'.join(sorted(directories)))


if __name__ == '__main__':
    main()
