#!/usr/bin/env python3
"""Check production dependency closures, including build dependencies."""
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    policy = json.loads((ROOT / 'scripts/rust-codec-policy.json').read_text(encoding='utf-8'))
    for rule in policy['rules']:
        # Check the current platform on each native CI runner. Do not require
        # unrelated platform packages to have been downloaded for metadata.
        tree = subprocess.run(
            ['cargo', 'tree', '--package', rule['package'], '--edges', 'normal,build',
             '--prefix', 'none', '--format', '{p}', '--locked', '--offline'],
            cwd=ROOT, check=True, capture_output=True, timeout=60,
        )
        names = {line.split()[0] for line in tree.stdout.decode('utf-8').splitlines() if line.strip()}
        if rule['package'] not in names:
            raise SystemExit('Missing policy root: ' + rule['package'])
        forbidden = names.intersection(rule.get('forbidden_runtime_dependencies', []))
        if forbidden:
            raise SystemExit(rule['package'] + ': forbidden dependencies reachable: ' + ', '.join(sorted(forbidden)))
        if 'allowed_local_dependencies' in rule:
            local = {name for name in names if name.startswith('xdzip-')} - {rule['package']}
            unexpected = local.difference(rule['allowed_local_dependencies'])
            if unexpected:
                raise SystemExit(rule['package'] + ': unexpected local dependencies: ' +
                                 ', '.join(sorted(unexpected)))
        print(rule['package'] + ': production dependency policy passed')


if __name__ == '__main__':
    main()
