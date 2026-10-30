#!/usr/bin/env python3
"""Classify complete Git trees conservatively; shadow never suppresses validation."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SHA = re.compile('[a-f0-9]{40}')
SCOPES = ('docs', 'foundation', 'qt')


def git(root, *arguments):
    result = subprocess.run(['git', *arguments], cwd=root, capture_output=True, timeout=30)
    if result.returncode:
        raise ValueError('git-read-failed')
    return result.stdout


def policy(root=ROOT):
    value = json.loads((root / 'scripts/ci-plan-policy.json').read_text(encoding='utf-8'))
    if (value['schema_version'] != 1 or value['mode'] not in ('shadow', 'enabled')
            or value['plain_documents'] != ['docs/README.md']
            or any(type(value[k]) is not int or not 0 < value[k] <= limit
                   for k, limit in [('max_files', 200), ('max_diff_bytes', 1048576),
                                    ('max_document_bytes', 131072)])):
        raise ValueError('invalid-policy')
    return value


def baseline(event, payload, env, root, revision):
    if event == 'push':
        if env.get('GITHUB_REF') != 'refs/heads/main':
            raise ValueError('tag-or-non-main-push')
        base = payload.get('before', '')
        if payload.get('after') != revision:
            raise ValueError('push-source-mismatch')
    elif event == 'pull_request':
        pr = payload['pull_request']
        if pr['head']['repo']['full_name'] != env.get('GITHUB_REPOSITORY'):
            raise ValueError('fork-pr')
        base, head = pr['base']['sha'], pr['head']['sha']
        if not SHA.fullmatch(head):
            raise ValueError('invalid-pr-head')
        parents = git(root, 'rev-list', '--parents', '-n', '1', revision).decode('ascii').split()
        if parents != [revision, base, head]:
            raise ValueError('unverified-pr-merge-tree')
    else:
        raise ValueError('explicit-or-unknown-event')
    if not SHA.fullmatch(base) or base == '0' * 40:
        raise ValueError('missing-base')
    if git(root, 'cat-file', '-t', base).strip() != b'commit':
        raise ValueError('missing-base')
    git(root, 'merge-base', '--is-ancestor', base, revision)
    return base


def changed_records(raw, config):
    if len(raw) > config['max_diff_bytes']:
        raise ValueError('large-diff')
    parts = raw.split(b'\0')
    if parts[-1] or len(parts) % 2 != 1:
        raise ValueError('incomplete-diff')
    records = []
    for index in range(0, len(parts) - 1, 2):
        header = parts[index].decode('ascii').split()
        name = parts[index + 1].decode('utf-8')
        if (len(header) != 5 or header[0] != ':100644' or header[1] != '100644'
                or header[4] != 'M' or any(not SHA.fullmatch(v) for v in header[2:4])):
            raise ValueError('added-deleted-renamed-or-mode-change')
        if (not name or name.startswith('/') or '\\' in name or ':' in name
                or any(ord(c) < 32 for c in name) or '..' in name.split('/')):
            raise ValueError('unrecognized-path')
        records.append((name, header[2], header[3]))
    if not records or len(records) > config['max_files'] or len({r[0] for r in records}) != len(records):
        raise ValueError('empty-large-or-duplicate-diff')
    return records


def plan(root=ROOT, env=None, payload=None, scope='docs'):
    env = os.environ if env is None else env
    result = {'schema_version': 1, 'scope': scope, 'revision': env.get('GITHUB_SHA', ''),
              'run_id': env.get('GITHUB_RUN_ID', ''), 'run_attempt': env.get('GITHUB_RUN_ATTEMPT', ''),
              'mode': 'shadow', 'classification': 'full', 'reason': 'classification-failed',
              'run_heavy': True, 'changed_files': None, 'base': None}
    try:
        config = policy(root)
        result['mode'] = config['mode']
        revision = result['revision']
        if not SHA.fullmatch(revision) or git(root, 'rev-parse', 'HEAD').decode('ascii').strip() != revision:
            raise ValueError('checkout-source-mismatch')
        if payload is None:
            payload = json.loads(Path(env['GITHUB_EVENT_PATH']).read_text(encoding='utf-8'))
        base = baseline(env.get('GITHUB_EVENT_NAME'), payload, env, root, revision)
        raw = git(root, 'diff', '--raw', '-z', '--no-renames', '--no-abbrev', base, revision, '--')
        records = changed_records(raw, config)
        result.update(base=base, changed_files=len(records))
        if any(name not in config['plain_documents'] for name, _, _ in records):
            raise ValueError('runtime-contract-or-unknown-path')
        for _, before, after in records:
            for blob in (before, after):
                content = git(root, 'cat-file', 'blob', blob)
                if len(content) > config['max_document_bytes'] or b'\0' in content:
                    raise ValueError('large-or-binary-document')
                content.decode('utf-8')
        result.update(classification='plain-docs', reason='explicit-navigation-document')
        result['run_heavy'] = config['mode'] == 'shadow'
    except (OSError, ValueError, KeyError, TypeError, UnicodeError, subprocess.SubprocessError) as error:
        # Never expose payloads, Git stderr, host paths or unclassified filenames.
        known = str(error) if type(error) is ValueError else ''
        result['reason'] = known if re.fullmatch('[a-z][a-z0-9-]{1,80}', known) else 'classification-read-failed'
        result.update(classification='full', run_heavy=True)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--scope', choices=SCOPES, required=True)
    args = parser.parse_args()
    result = plan(scope=args.scope)
    target = ROOT / 'build/ci-plans' / (args.scope + '-plan.json')
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    if output := os.environ.get('GITHUB_OUTPUT'):
        with Path(output).open('a', encoding='utf-8') as file:
            file.write('run_heavy=' + str(result['run_heavy']).lower() + '\n')
    print(json.dumps(result))


if __name__ == '__main__':
    main()
